//! Addition and subtraction operators.

use crate::{
    add,
    arch::word::{Word, DoubleWord},
    buffer::{Buffer, TypedRepr::*, TypedReprRef::*},
    helper_macros,
    ibig::IBig,
    primitive::split_double_word,
    sign::Sign::*,
    ubig::UBig,
};
use core::{
    mem,
    ops::{Add, AddAssign, Sub, SubAssign},
};

impl Add<UBig> for UBig {
    type Output = UBig;

    #[inline]
    fn add(self, rhs: UBig) -> UBig {
        ubig::add_repr_val_val(self.into_repr(), rhs.into_repr())
    }
}

impl Add<&UBig> for UBig {
    type Output = UBig;

    #[inline]
    fn add(self, rhs: &UBig) -> UBig {
        ubig::add_repr_ref_val(rhs.repr(), self.into_repr())
    }
}

impl Add<UBig> for &UBig {
    type Output = UBig;

    #[inline]
    fn add(self, rhs: UBig) -> UBig {
        ubig::add_repr_ref_val(self.repr(), rhs.into_repr())
    }
}

impl Add<&UBig> for &UBig {
    type Output = UBig;

    #[inline]
    fn add(self, rhs: &UBig) -> UBig {
        ubig::add_repr_ref_ref(self.repr(), rhs.repr())
    }
}

impl AddAssign<UBig> for UBig {
    #[inline]
    fn add_assign(&mut self, rhs: UBig) {
        *self = mem::take(self) + rhs;
    }
}

impl AddAssign<&UBig> for UBig {
    #[inline]
    fn add_assign(&mut self, rhs: &UBig) {
        *self = mem::take(self) + rhs;
    }
}

impl Sub<UBig> for UBig {
    type Output = UBig;

    #[inline]
    fn sub(self, rhs: UBig) -> UBig {
        ubig::sub_repr_val_val(self.into_repr(), rhs.into_repr())
    }
}

impl Sub<&UBig> for UBig {
    type Output = UBig;

    #[inline]
    fn sub(self, rhs: &UBig) -> UBig {
        ubig::sub_repr_val_ref(self.into_repr(), rhs.repr())
    }
}

impl Sub<UBig> for &UBig {
    type Output = UBig;

    #[inline]
    fn sub(self, rhs: UBig) -> UBig {
        ubig::sub_repr_ref_val(self.repr(), rhs.into_repr())
    }
}

impl Sub<&UBig> for &UBig {
    type Output = UBig;

    #[inline]
    fn sub(self, rhs: &UBig) -> UBig {
        match (self.repr(), rhs.repr()) {
            (RefSmall(dword0), RefSmall(dword1)) => ubig::sub_dword(dword0, dword1),
            (RefSmall(_), RefLarge(_)) => UBig::panic_negative(),
            (RefLarge(buffer0), RefSmall(dword1)) => ubig::sub_large_dword(buffer0.into(), dword1),
            (RefLarge(buffer0), RefLarge(buffer1)) => ubig::sub_large(buffer0.into(), buffer1),
        }
    }
}

impl SubAssign<UBig> for UBig {
    #[inline]
    fn sub_assign(&mut self, rhs: UBig) {
        *self = mem::take(self) - rhs;
    }
}

impl SubAssign<&UBig> for UBig {
    #[inline]
    fn sub_assign(&mut self, rhs: &UBig) {
        *self = mem::take(self) - rhs;
    }
}

impl Add<IBig> for IBig {
    type Output = IBig;

    #[inline]
    fn add(self, rhs: IBig) -> IBig {
        let (sign0, mag0) = self.into_sign_repr();
        let (sign1, mag1) = rhs.into_sign_repr();
        match (sign0, sign1) {
            (Positive, Positive) => IBig::from(ubig::add_repr_val_val(mag0, mag1)),
            (Positive, Negative) => ibig::sub_repr_val_val(mag0, mag1),
            (Negative, Positive) => ibig::sub_repr_val_val(mag1, mag0),
            (Negative, Negative) => -IBig::from(ubig::add_repr_val_val(mag0, mag1)),
        }
    }
}

impl Add<&IBig> for IBig {
    type Output = IBig;

    #[inline]
    fn add(self, rhs: &IBig) -> IBig {
        let (sign0, mag0) = self.into_sign_repr();
        let (sign1, mag1) = rhs.signed_repr();
        match (sign0, sign1) {
            (Positive, Positive) => IBig::from(ubig::add_repr_ref_val(mag1, mag0)),
            (Positive, Negative) => -ibig::sub_repr_ref_val(mag1, mag0),
            (Negative, Positive) => ibig::sub_repr_ref_val(mag1, mag0),
            (Negative, Negative) => -IBig::from(ubig::add_repr_ref_val(mag1, mag0)),
        }
    }
}

impl Add<IBig> for &IBig {
    type Output = IBig;

    #[inline]
    fn add(self, rhs: IBig) -> IBig {
        rhs.add(self)
    }
}

impl Add<&IBig> for &IBig {
    type Output = IBig;

    #[inline]
    fn add(self, rhs: &IBig) -> IBig {
        let (sign0, mag0) = self.signed_repr();
        let (sign1, mag1) = rhs.signed_repr();
        match (sign0, sign1) {
            (Positive, Positive) => IBig::from(ubig::add_repr_ref_ref(mag0, mag1)),
            (Positive, Negative) => ibig::sub_repr_ref_ref(mag0, mag1),
            (Negative, Positive) => ibig::sub_repr_ref_ref(mag1, mag0),
            (Negative, Negative) => -IBig::from(ubig::add_repr_ref_ref(mag0, mag1)),
        }
    }
}

impl AddAssign<IBig> for IBig {
    #[inline]
    fn add_assign(&mut self, rhs: IBig) {
        *self = mem::take(self) + rhs;
    }
}

impl AddAssign<&IBig> for IBig {
    #[inline]
    fn add_assign(&mut self, rhs: &IBig) {
        *self = mem::take(self) + rhs;
    }
}

impl Sub<IBig> for IBig {
    type Output = IBig;

    #[inline]
    fn sub(self, rhs: IBig) -> IBig {
        self + -rhs
    }
}

impl Sub<&IBig> for IBig {
    type Output = IBig;

    #[inline]
    fn sub(self, rhs: &IBig) -> IBig {
        -(-self + rhs)
    }
}

impl Sub<IBig> for &IBig {
    type Output = IBig;

    #[inline]
    fn sub(self, rhs: IBig) -> IBig {
        self + -rhs
    }
}

impl Sub<&IBig> for &IBig {
    type Output = IBig;

    #[inline]
    fn sub(self, rhs: &IBig) -> IBig {
        let (sign0, mag0) = self.signed_repr();
        let (sign1, mag1) = rhs.signed_repr();
        match (sign0, sign1) {
            (Positive, Positive) => ibig::sub_repr_ref_ref(mag0, mag1),
            (Positive, Negative) => IBig::from(ubig::add_repr_ref_ref(mag0, mag1)),
            (Negative, Positive) => -IBig::from(ubig::add_repr_ref_ref(mag0, mag1)),
            (Negative, Negative) => ibig::sub_repr_ref_ref(mag1, mag0),
        }
    }
}

impl SubAssign<IBig> for IBig {
    #[inline]
    fn sub_assign(&mut self, rhs: IBig) {
        *self = mem::take(self) - rhs;
    }
}

impl SubAssign<&IBig> for IBig {
    #[inline]
    fn sub_assign(&mut self, rhs: &IBig) {
        *self = mem::take(self) - rhs;
    }
}

macro_rules! impl_add_ubig_unsigned {
    ($t:ty) => {
        impl Add<$t> for UBig {
            type Output = UBig;

            #[inline]
            fn add(self, rhs: $t) -> UBig {
                self + UBig::from_unsigned(rhs)
            }
        }

        impl Add<$t> for &UBig {
            type Output = UBig;

            #[inline]
            fn add(self, rhs: $t) -> UBig {
                self + UBig::from_unsigned(rhs)
            }
        }

        helper_macros::forward_binop_second_arg_by_value!(impl Add<$t> for UBig, add);
        helper_macros::forward_binop_swap_args!(impl Add<UBig> for $t, add);

        impl AddAssign<$t> for UBig {
            #[inline]
            fn add_assign(&mut self, rhs: $t) {
                *self += UBig::from_unsigned(rhs)
            }
        }

        helper_macros::forward_binop_assign_arg_by_value!(impl AddAssign<$t> for UBig, add_assign);

        impl Sub<$t> for UBig {
            type Output = UBig;

            #[inline]
            fn sub(self, rhs: $t) -> UBig {
                self - UBig::from_unsigned(rhs)
            }
        }

        impl Sub<$t> for &UBig {
            type Output = UBig;

            #[inline]
            fn sub(self, rhs: $t) -> UBig {
                self - UBig::from_unsigned(rhs)
            }
        }

        helper_macros::forward_binop_second_arg_by_value!(impl Sub<$t> for UBig, sub);

        impl SubAssign<$t> for UBig {
            #[inline]
            fn sub_assign(&mut self, rhs: $t) {
                *self -= UBig::from_unsigned(rhs)
            }
        }

        helper_macros::forward_binop_assign_arg_by_value!(impl SubAssign<$t> for UBig, sub_assign);
    };
}

impl_add_ubig_unsigned!(u8);
impl_add_ubig_unsigned!(u16);
impl_add_ubig_unsigned!(u32);
impl_add_ubig_unsigned!(u64);
impl_add_ubig_unsigned!(u128);
impl_add_ubig_unsigned!(usize);

macro_rules! impl_add_ubig_signed {
    ($t:ty) => {
        impl Add<$t> for UBig {
            type Output = UBig;

            #[inline]
            fn add(self, rhs: $t) -> UBig {
                UBig::from_ibig(IBig::from(self) + IBig::from(rhs))
            }
        }

        impl Add<$t> for &UBig {
            type Output = UBig;

            #[inline]
            fn add(self, rhs: $t) -> UBig {
                UBig::from_ibig(IBig::from(self) + IBig::from(rhs))
            }
        }

        helper_macros::forward_binop_second_arg_by_value!(impl Add<$t> for UBig, add);
        helper_macros::forward_binop_swap_args!(impl Add<UBig> for $t, add);

        impl AddAssign<$t> for UBig {
            #[inline]
            fn add_assign(&mut self, rhs: $t) {
                *self = mem::take(self) + rhs
            }
        }

        helper_macros::forward_binop_assign_arg_by_value!(impl AddAssign<$t> for UBig, add_assign);

        impl Sub<$t> for UBig {
            type Output = UBig;

            #[inline]
            fn sub(self, rhs: $t) -> UBig {
                UBig::from_ibig(IBig::from(self) - IBig::from(rhs))
            }
        }

        impl Sub<$t> for &UBig {
            type Output = UBig;

            #[inline]
            fn sub(self, rhs: $t) -> UBig {
                UBig::from_ibig(IBig::from(self) - IBig::from(rhs))
            }
        }

        helper_macros::forward_binop_second_arg_by_value!(impl Sub<$t> for UBig, sub);

        impl SubAssign<$t> for UBig {
            #[inline]
            fn sub_assign(&mut self, rhs: $t) {
                *self = mem::take(self) - rhs
            }
        }

        helper_macros::forward_binop_assign_arg_by_value!(impl SubAssign<$t> for UBig, sub_assign);
    };
}

impl_add_ubig_signed!(i8);
impl_add_ubig_signed!(i16);
impl_add_ubig_signed!(i32);
impl_add_ubig_signed!(i64);
impl_add_ubig_signed!(i128);
impl_add_ubig_signed!(isize);

macro_rules! impl_add_ibig_primitive {
    ($t:ty) => {
        impl Add<$t> for IBig {
            type Output = IBig;

            #[inline]
            fn add(self, rhs: $t) -> IBig {
                self + IBig::from(rhs)
            }
        }

        impl Add<$t> for &IBig {
            type Output = IBig;

            #[inline]
            fn add(self, rhs: $t) -> IBig {
                self + IBig::from(rhs)
            }
        }

        helper_macros::forward_binop_second_arg_by_value!(impl Add<$t> for IBig, add);
        helper_macros::forward_binop_swap_args!(impl Add<IBig> for $t, add);

        impl AddAssign<$t> for IBig {
            #[inline]
            fn add_assign(&mut self, rhs: $t) {
                *self += IBig::from(rhs)
            }
        }

        helper_macros::forward_binop_assign_arg_by_value!(impl AddAssign<$t> for IBig, add_assign);

        impl Sub<$t> for IBig {
            type Output = IBig;

            #[inline]
            fn sub(self, rhs: $t) -> IBig {
                self - IBig::from(rhs)
            }
        }

        impl Sub<$t> for &IBig {
            type Output = IBig;

            #[inline]
            fn sub(self, rhs: $t) -> IBig {
                self - IBig::from(rhs)
            }
        }

        impl Sub<IBig> for $t {
            type Output = IBig;

            #[inline]
            fn sub(self, rhs: IBig) -> IBig {
                IBig::from(rhs) - self
            }
        }

        impl Sub<&IBig> for $t {
            type Output = IBig;

            #[inline]
            fn sub(self, rhs: &IBig) -> IBig {
                rhs - IBig::from(self)
            }
        }

        helper_macros::forward_binop_second_arg_by_value!(impl Sub<$t> for IBig, sub);
        helper_macros::forward_binop_first_arg_by_value!(impl Sub<IBig> for $t, sub);

        impl SubAssign<$t> for IBig {
            #[inline]
            fn sub_assign(&mut self, rhs: $t) {
                *self -= IBig::from(rhs)
            }
        }

        helper_macros::forward_binop_assign_arg_by_value!(impl SubAssign<$t> for IBig, sub_assign);
    };
}

impl_add_ibig_primitive!(u8);
impl_add_ibig_primitive!(u16);
impl_add_ibig_primitive!(u32);
impl_add_ibig_primitive!(u64);
impl_add_ibig_primitive!(u128);
impl_add_ibig_primitive!(usize);
impl_add_ibig_primitive!(i8);
impl_add_ibig_primitive!(i16);
impl_add_ibig_primitive!(i32);
impl_add_ibig_primitive!(i64);
impl_add_ibig_primitive!(i128);
impl_add_ibig_primitive!(isize);

mod ubig {
    use crate::buffer::{TypedRepr, TypedReprRef};
    use super::*;

    #[inline]
    pub fn add_repr_ref_ref(lhs: TypedReprRef, rhs: TypedReprRef) -> UBig {
        match (lhs, rhs) {
            (RefSmall(dword0), RefSmall(dword1)) => ubig::add_dword(dword0, dword1),
            (RefSmall(dword0), RefLarge(buffer1)) => ubig::add_large_dword(buffer1.into(), dword0),
            (RefLarge(buffer0), RefSmall(dword1)) => ubig::add_large_dword(buffer0.into(), dword1),
            (RefLarge(buffer0), RefLarge(buffer1)) => {
                if buffer0.len() >= buffer1.len() {
                    ubig::add_large(buffer0.into(), buffer1)
                } else {
                    ubig::add_large(buffer1.into(), buffer0)
                }
            },
        }
    }

    #[inline]
    pub fn add_repr_ref_val(lhs: TypedReprRef, rhs: TypedRepr) -> UBig {
        match (lhs, rhs) {
            (RefSmall(dword0), Small(dword1)) => ubig::add_dword(dword0, dword1),
            (RefSmall(dword0), Large(buffer1)) => ubig::add_large_dword(buffer1, dword0),
            (RefLarge(buffer0), Small(dword1)) => ubig::add_large_dword(buffer0.into(), dword1),
            (RefLarge(buffer0), Large(buffer1)) => ubig::add_large(buffer1, buffer0),
        }
    }

    #[inline]
    pub fn add_repr_val_val(lhs: TypedRepr, rhs: TypedRepr) -> UBig {
        match (lhs, rhs) {
            (Small(dword0), Small(dword1)) => add_dword(dword0, dword1),
            (Small(dword0), Large(buffer1)) => add_large_dword(buffer1, dword0),
            (Large(buffer0), Small(dword1)) => add_large_dword(buffer0, dword1),
            (Large(buffer0), Large(buffer1)) => {
                if buffer0.len() >= buffer1.len() {
                    add_large(buffer0, &buffer1)
                } else {
                    add_large(buffer1, &buffer0)
                }
            },
        }
    }
    
    #[inline]
    pub fn add_dword(a: DoubleWord, b: DoubleWord) -> UBig {
        let (res, overflow) = a.overflowing_add(b);
        if overflow {
            let (lo, hi) = split_double_word(res);
            let mut buffer = Buffer::allocate(3);
            buffer.push(lo);
            buffer.push(hi);
            buffer.push(1);
            buffer.into()
        } else {
            res.into()
        }
    }

    #[inline]
    pub fn add_large_dword(mut buffer: Buffer, rhs: DoubleWord) -> UBig {
        debug_assert!(buffer.len() >= 3);
        if add::add_dword_in_place(&mut buffer, rhs) {
            buffer.push_may_reallocate(1);
        }
        buffer.into()
    }

    #[inline]
    pub fn add_large(mut buffer: Buffer, rhs: &[Word]) -> UBig {
        let n = buffer.len().min(rhs.len());
        let overflow = add::add_same_len_in_place(&mut buffer[..n], &rhs[..n]);
        if rhs.len() > n {
            buffer.ensure_capacity(rhs.len());
            buffer.push_slice(&rhs[n..]);
        }
        if overflow && add::add_one_in_place(&mut buffer[n..]) {
            buffer.push_may_reallocate(1);
        }
        buffer.into()
    }

    #[inline]
    pub fn sub_repr_val_val(lhs: TypedRepr, rhs: TypedRepr) -> UBig {
        match (lhs, rhs) {
            (Small(dword0), Small(dword1)) => sub_dword(dword0, dword1),
            (Small(_), Large(_)) => UBig::panic_negative(),
            (Large(buffer0), Small(dword1)) => sub_large_dword(buffer0, dword1),
            (Large(buffer0), Large(buffer1)) => sub_large(buffer0, &buffer1),
        }
    }

    #[inline]
    pub fn sub_repr_val_ref(lhs: TypedRepr, rhs: TypedReprRef) -> UBig {
        match (lhs, rhs) {
            (Small(dword0), RefSmall(dword1)) => sub_dword(dword0, dword1),
            (Small(_), RefLarge(_)) => UBig::panic_negative(),
            (Large(buffer0), RefSmall(dword1)) => sub_large_dword(buffer0, dword1),
            (Large(buffer0), RefLarge(buffer1)) => sub_large(buffer0, buffer1),
        }
    }

    #[inline]
    pub fn sub_repr_ref_val(lhs: TypedReprRef, rhs: TypedRepr) -> UBig {
        match (lhs, rhs) {
            (RefSmall(dword0), Small(dword1)) => sub_dword(dword0, dword1),
            (RefSmall(_), Large(_)) => UBig::panic_negative(),
            (RefLarge(buffer0), Small(dword1)) => sub_large_dword(buffer0.into(), dword1),
            (RefLarge(buffer0), Large(buffer1)) => sub_large_ref_val(buffer0, buffer1.into()),
        }
    }

    #[inline]
    pub fn sub_dword(a: DoubleWord, b: DoubleWord) -> UBig {
        match a.checked_sub(b) {
            Some(res) => res.into(),
            None => UBig::panic_negative(),
        }
    }

    #[inline]
    pub fn sub_large_dword(mut lhs: Buffer, rhs: DoubleWord) -> UBig {
        let overflow = add::sub_dword_in_place(&mut lhs, rhs);
        assert!(!overflow);
        lhs.into()
    }

    #[inline]
    pub fn sub_large(mut lhs: Buffer, rhs: &[Word]) -> UBig {
        if lhs.len() < rhs.len() || add::sub_in_place(&mut lhs, rhs) {
            UBig::panic_negative();
        }
        lhs.into()
    }
    
    pub fn sub_large_ref_val(lhs: &[Word], mut rhs: Buffer) -> UBig {
        let n = rhs.len();
        if lhs.len() < n {
            UBig::panic_negative();
        }
        let borrow = add::sub_same_len_in_place_swap(&lhs[..n], &mut rhs);
        rhs.ensure_capacity(lhs.len());
        rhs.push_slice(&lhs[n..]);
        if borrow && add::sub_one_in_place(&mut rhs[n..]) {
            UBig::panic_negative();
        }
        rhs.into()
    }
}

mod ibig {
    use crate::buffer::{TypedReprRef, TypedRepr};
    use super::*;

    #[inline]
    pub fn sub_repr_val_val(lhs: TypedRepr, rhs: TypedRepr) -> IBig {
        match (lhs, rhs) {
            (Small(dword0), Small(dword1)) => sub_dword_word(dword0, dword1),
            (Small(dword0), Large(buffer1)) => -sub_large_dword(buffer1, dword0),
            (Large(buffer0), Small(dword1)) => sub_large_dword(buffer0, dword1),
            (Large(buffer0), Large(buffer1)) => {
                if buffer0.len() >= buffer1.len() {
                    sub_large(buffer0, &buffer1)
                } else {
                    -sub_large(buffer1, &buffer0)
                }
            },
        }
    }

    #[inline]
    pub fn sub_repr_ref_val(lhs: TypedReprRef, rhs: TypedRepr) -> IBig {
        match (lhs, rhs) {
            (RefSmall(dword0), Small(dword1)) => sub_dword_word(dword0, dword1),
            (RefSmall(dword0), Large(buffer1)) => -sub_large_dword(buffer1, dword0),
            (RefLarge(buffer0), Small(dword1)) => sub_large_dword(buffer0.into(), dword1),
            (RefLarge(buffer0), Large(buffer1)) => -sub_large(buffer1, buffer0),
        }
    }

    #[inline]
    pub fn sub_repr_ref_ref(lhs: TypedReprRef, rhs: TypedReprRef) -> IBig {
        match (lhs, rhs) {
            (RefSmall(dword0), RefSmall(dword1)) => sub_dword_word(dword0, dword1),
            (RefSmall(dword0), RefLarge(buffer1)) => -sub_large_dword(buffer1.into(), dword0),
            (RefLarge(buffer0), RefSmall(dword1)) => sub_large_dword(buffer0.into(), dword1),
            (RefLarge(buffer0), RefLarge(buffer1)) => {
                if buffer0.len() >= buffer1.len() {
                    sub_large(buffer0.into(), buffer1)
                } else {
                    -sub_large(buffer1.into(), buffer0)
                }
            },
        }
    }

    #[inline]
    pub fn sub_dword_word(lhs: DoubleWord, rhs: DoubleWord) -> IBig {
        let (val, overflow) = lhs.overflowing_sub(rhs);
        if !overflow {
            IBig::from(val)
        } else {
            -IBig::from(val.wrapping_neg())
        }
    }

    #[inline]
    pub fn sub_large_dword(lhs: Buffer, rhs: DoubleWord) -> IBig {
        ubig::sub_large_dword(lhs, rhs).into()
    }

    #[inline]
    pub fn sub_large(mut lhs: Buffer, rhs: &[Word]) -> IBig {
        if lhs.len() >= rhs.len() {
            let sign = add::sub_in_place_with_sign(&mut lhs, rhs);
            IBig::from_sign_magnitude(sign, lhs.into())
        } else {
            let res = ubig::sub_large_ref_val(rhs, lhs);
            IBig::from_sign_magnitude(Negative, res)
        }
    }
}
