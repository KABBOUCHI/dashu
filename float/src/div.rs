use crate::{
    error::check_inf_operands,
    fbig::FBig,
    helper_macros,
    repr::{Context, Repr, Word},
    round::{Round, Rounded},
    utils::{digit_len, shl_digits_in_place},
};
use core::ops::{Div, DivAssign};
use dashu_base::{Approximation, DivRem};
use dashu_int::{IBig, UBig};

impl<R: Round, const B: Word> Div<FBig<R, B>> for FBig<R, B> {
    type Output = FBig<R, B>;
    fn div(self, rhs: FBig<R, B>) -> Self::Output {
        let context = Context::max(self.context, rhs.context);
        FBig::new_raw(context.repr_div(self.repr, &rhs.repr).value(), context)
    }
}

impl<'l, const B: Word, R: Round> Div<FBig<R, B>> for &'l FBig<R, B> {
    type Output = FBig<R, B>;
    fn div(self, rhs: FBig<R, B>) -> Self::Output {
        let context = Context::max(self.context, rhs.context);
        FBig::new_raw(context.repr_div(self.repr.clone(), &rhs.repr).value(), context)
    }
}

impl<'r, const B: Word, R: Round> Div<&'r FBig<R, B>> for FBig<R, B> {
    type Output = Self;
    fn div(self, rhs: &FBig<R, B>) -> Self::Output {
        let context = Context::max(self.context, rhs.context);
        FBig::new_raw(context.repr_div(self.repr, &rhs.repr).value(), context)
    }
}

impl<'l, 'r, const B: Word, R: Round> Div<&'r FBig<R, B>> for &'l FBig<R, B> {
    type Output = FBig<R, B>;
    fn div(self, rhs: &FBig<R, B>) -> Self::Output {
        let context = Context::max(self.context, rhs.context);
        FBig::new_raw(context.repr_div(self.repr.clone(), &rhs.repr).value(), context)
    }
}

impl<R: Round, const B: Word> DivAssign for FBig<R, B> {
    #[inline]
    fn div_assign(&mut self, rhs: Self) {
        *self = core::mem::take(self) / rhs
    }
}
impl<R: Round, const B: Word> DivAssign<&FBig<R, B>> for FBig<R, B> {
    #[inline]
    fn div_assign(&mut self, rhs: &FBig<R, B>) {
        *self = core::mem::take(self) / rhs
    }
}

macro_rules! impl_add_sub_primitive_with_fbig {
    ($($t:ty)*) => {$(
        helper_macros::impl_commutative_binop_with_primitive!(impl Div<$t>, div);
        helper_macros::impl_binop_assign_with_primitive!(impl DivAssign<$t>, div_assign);
    )*};
}
impl_add_sub_primitive_with_fbig!(u8 u16 u32 u64 u128 usize UBig i8 i16 i32 i64 i128 isize IBig);

impl<R: Round, const B: Word> FBig<R, B> {
    /// Create a floating number by dividing two integers with given precision
    #[inline]
    #[deprecated] // TODO: remove this, implement as From<RBig> in future
    pub fn from_ratio(numerator: IBig, denominator: IBig, precision: usize) -> Self {
        let context = Context::new(precision);
        let dummy = Context::new(0);
        let n = FBig::new_raw(Repr::new(numerator, 0), dummy);
        let d = FBig::new_raw(Repr::new(denominator, 0), dummy);
        context.div(&n, &d).value()
    }
}

impl<R: Round> Context<R> {
    pub(crate) fn repr_div<const B: Word>(&self, lhs: Repr<B>, rhs: &Repr<B>) -> Rounded<Repr<B>> {
        check_inf_operands(&lhs, &rhs);

        // this method don't deal with the case where lhs significand is too large
        debug_assert!(lhs.digits() <= self.precision + rhs.digits());

        let (mut q, mut r) = lhs.significand.div_rem(&rhs.significand);
        let mut e = lhs.exponent - rhs.exponent;
        if r.is_zero() {
            return Approximation::Exact(Repr::new(q, e));
        }

        let ddigits = digit_len::<B>(&rhs.significand);
        if q.is_zero() {
            // lhs.significand < rhs.significand
            let rdigits = digit_len::<B>(&r); // rdigits <= ddigits
            let shift = ddigits + self.precision - rdigits;
            shl_digits_in_place::<B>(&mut r, shift);
            e -= shift as isize;
            let (q0, r0) = r.div_rem(&rhs.significand);
            q = q0;
            r = r0;
        } else {
            let ndigits = digit_len::<B>(&q) + ddigits;
            if ndigits < ddigits + self.precision {
                // TODO: here the operations can be optimized: 1. prevent double power, 2. q += q0 can be |= if B is power of 2
                let shift = ddigits + self.precision - ndigits;
                shl_digits_in_place::<B>(&mut q, shift);
                shl_digits_in_place::<B>(&mut r, shift);
                e -= shift as isize;

                let (q0, r0) = r.div_rem(&rhs.significand);
                q += q0;
                r = r0;
            }
        }

        if r.is_zero() {
            Approximation::Exact(Repr::new(q, e))
        } else {
            let adjust = R::round_ratio(&q, r, &rhs.significand);
            Approximation::Inexact(Repr::new(q + adjust, e), adjust)
        }
    }

    pub fn div<const B: Word>(&self, lhs: &FBig<R, B>, rhs: &FBig<R, B>) -> Rounded<FBig<R, B>> {
        let lhs_repr = if !lhs.repr.is_zero()
            && lhs.repr.digits_ub() > rhs.repr.digits_lb() + self.precision
        {
            // shrink lhs if it's larger than necessary
            Self::new(rhs.repr.digits() + self.precision)
                .repr_round(lhs.repr.clone())
                .value()
        } else {
            lhs.repr.clone()
        };
        self.repr_div(lhs_repr, &rhs.repr)
            .map(|v| FBig::new_raw(v, *self))
    }
}

// TODO: implement div_euclid, rem_euclid, div_rem_euclid for float, as it can be properly defined
//       maybe also implement rem and div_rem to be consistent with the builtin float
