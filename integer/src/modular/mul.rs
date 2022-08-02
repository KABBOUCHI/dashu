use crate::{
    add,
    arch::word::Word,
    bits::locate_top_word_plus_one,
    cmp, div,
    error::panic_different_rings,
    helper_macros::debug_assert_zero,
    math,
    memory::{self, Memory, MemoryAllocation},
    modular::{
        modulo::{Modulo, ModuloRepr, ModuloSingleRaw},
        modulo_ring::{ModuloRingLarge, ModuloRingSingle},
    },
    mul,
    primitive::extend_word,
    shift,
    sign::Sign::Positive,
};
use alloc::alloc::Layout;
use core::ops::{Mul, MulAssign};

use super::{
    modulo::{ModuloDoubleRaw, ModuloLargeRaw},
    modulo_ring::ModuloRingDouble,
};

impl<'a> Mul<Modulo<'a>> for Modulo<'a> {
    type Output = Modulo<'a>;

    #[inline]
    fn mul(self, rhs: Modulo<'a>) -> Modulo<'a> {
        self.mul(&rhs)
    }
}

impl<'a> Mul<&Modulo<'a>> for Modulo<'a> {
    type Output = Modulo<'a>;

    #[inline]
    fn mul(mut self, rhs: &Modulo<'a>) -> Modulo<'a> {
        self.mul_assign(rhs);
        self
    }
}

impl<'a> Mul<Modulo<'a>> for &Modulo<'a> {
    type Output = Modulo<'a>;

    #[inline]
    fn mul(self, rhs: Modulo<'a>) -> Modulo<'a> {
        rhs.mul(self)
    }
}

impl<'a> Mul<&Modulo<'a>> for &Modulo<'a> {
    type Output = Modulo<'a>;

    #[inline]
    fn mul(self, rhs: &Modulo<'a>) -> Modulo<'a> {
        self.clone().mul(rhs)
    }
}

impl<'a> MulAssign<Modulo<'a>> for Modulo<'a> {
    #[inline]
    fn mul_assign(&mut self, rhs: Modulo<'a>) {
        self.mul_assign(&rhs)
    }
}

impl<'a> MulAssign<&Modulo<'a>> for Modulo<'a> {
    #[inline]
    fn mul_assign(&mut self, rhs: &Modulo<'a>) {
        match (self.repr_mut(), rhs.repr()) {
            (ModuloRepr::Single(raw0, ring), ModuloRepr::Single(raw1, ring1)) => {
                Modulo::check_same_ring_single(ring, ring1);
                *raw0 = ring.mul(*raw0, *raw1);
            }
            (ModuloRepr::Double(raw0, ring), ModuloRepr::Double(raw1, ring1)) => {
                Modulo::check_same_ring_double(ring, ring1);
                *raw0 = ring.mul(*raw0, *raw1);
            }
            (ModuloRepr::Large(raw0, ring), ModuloRepr::Large(raw1, ring1)) => {
                Modulo::check_same_ring_large(ring, ring1);
                let memory_requirement = ring.mul_memory_requirement();
                let mut allocation = MemoryAllocation::new(memory_requirement);
                ring.mul_in_place(raw0, raw1, &mut allocation.memory());
            }
            _ => panic_different_rings(),
        }
    }
}

impl ModuloRingSingle {
    #[inline]
    pub const fn mul(&self, lhs: ModuloSingleRaw, rhs: ModuloSingleRaw) -> ModuloSingleRaw {
        let product = extend_word(lhs.0 >> self.shift()) * extend_word(rhs.0);
        let (_, rem) = self.fast_div().div_rem(product);
        ModuloSingleRaw(rem)
    }

    #[inline]
    pub const fn sqr(&self, raw: ModuloSingleRaw) -> ModuloSingleRaw {
        let product = (extend_word(raw.0) * extend_word(raw.0)) >> self.shift();
        let (_, rem) = self.fast_div().div_rem(product);
        ModuloSingleRaw(rem)
    }
}

impl ModuloRingDouble {
    #[inline]
    pub const fn mul(&self, lhs: ModuloDoubleRaw, rhs: ModuloDoubleRaw) -> ModuloDoubleRaw {
        let (prod0, prod1) = math::mul_add_carry_dword(lhs.0 >> self.shift(), rhs.0, 0);
        let (_, rem) = self.fast_div().div_rem_double(prod0, prod1);
        ModuloDoubleRaw(rem)
    }

    #[inline]
    pub const fn sqr(&self, raw: ModuloDoubleRaw) -> ModuloDoubleRaw {
        let (prod0, prod1) = math::mul_add_carry_dword(raw.0 >> self.shift(), raw.0, 0);
        let (_, rem) = self.fast_div().div_rem_double(prod0, prod1);
        ModuloDoubleRaw(rem)
    }
}

impl ModuloRingLarge {
    pub(crate) fn mul_memory_requirement(&self) -> Layout {
        let n = self.normalized_modulus().len();
        memory::add_layout(
            memory::array_layout::<Word>(2 * n),
            memory::max_layout(
                mul::memory_requirement_exact(2 * n, n),
                div::memory_requirement_exact(2 * n, n),
            ),
        )
    }

    /// Returns a * b allocated in memory.
    pub(crate) fn mul_normalized<'a>(
        &self,
        a: &[Word],
        b: &[Word],
        memory: &'a mut Memory,
    ) -> &'a [Word] {
        let modulus = self.normalized_modulus();
        let n = modulus.len();
        debug_assert!(a.len() == n && b.len() == n);

        // trim the leading zeros in a, b
        let na = locate_top_word_plus_one(a);
        let nb = locate_top_word_plus_one(b);

        // product = (a * b >> self.shift())
        let (product, mut memory) = memory.allocate_slice_fill::<Word>(n.max(na + nb), 0);
        mul::multiply(&mut product[..na + nb], &a[..na], &b[..nb], &mut memory);
        debug_assert_zero!(shift::shr_in_place(product, self.shift()));

        // return product % normalized_modulus
        if na + nb > n {
            let _overflow =
                div::div_rem_in_place(product, modulus, self.fast_div_top(), &mut memory);
            &product[..n]
        } else {
            if cmp::cmp_same_len(product, modulus).is_ge() {
                debug_assert_zero!(add::sub_same_len_in_place(product, modulus));
            }
            product
        }
    }

    /// self *= rhs
    pub(crate) fn mul_in_place(
        &self,
        lhs: &mut ModuloLargeRaw,
        rhs: &ModuloLargeRaw,
        memory: &mut Memory,
    ) {
        // TODO: truncate leading zeros before mul
        let prod = self.mul_normalized(&lhs.0, &rhs.0, memory);
        lhs.0.copy_from_slice(prod)
    }

    pub(crate) fn sqr_in_place(&self, raw: &mut ModuloLargeRaw, memory: &mut Memory) {
        // TODO: truncate leading zeros before sqr
        // TODO(next): use specialized square function
        let prod = self.mul_normalized(&raw.0, &raw.0, memory);
        raw.0.copy_from_slice(prod)
    }
}
