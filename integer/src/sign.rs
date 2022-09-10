//! Operators on the sign of [IBig].

use crate::{
    ibig::IBig,
    ops::{Abs, UnsignedAbs},
    ubig::UBig,
};
use core::ops::{Mul, MulAssign, Neg};
use dashu_base::Sign;

impl IBig {
    /// A number representing the sign of `self`.
    ///
    /// * -1 if the number is negative
    /// * 0 if the number is zero
    /// * 1 if the number is positive
    ///
    /// # Examples
    /// ```
    /// # use dashu_int::IBig;
    /// assert_eq!(IBig::from(-500).signum(), IBig::from(-1));
    /// ```
    #[inline]
    pub const fn signum(&self) -> IBig {
        IBig(self.0.signum())
    }
}

impl Neg for UBig {
    type Output = IBig;

    #[inline]
    fn neg(self) -> IBig {
        IBig(self.0.neg())
    }
}

impl Neg for IBig {
    type Output = IBig;

    #[inline]
    fn neg(self) -> IBig {
        IBig(self.0.neg())
    }
}

impl Neg for &UBig {
    type Output = IBig;

    #[inline]
    fn neg(self) -> IBig {
        IBig(self.0.clone().neg())
    }
}

impl Neg for &IBig {
    type Output = IBig;

    #[inline]
    fn neg(self) -> IBig {
        IBig(self.0.clone().neg())
    }
}

impl Abs for IBig {
    type Output = IBig;

    #[inline]
    fn abs(self) -> IBig {
        IBig(self.0.with_sign(Sign::Positive))
    }
}

impl Abs for &IBig {
    type Output = IBig;

    #[inline]
    fn abs(self) -> IBig {
        IBig(self.0.clone().with_sign(Sign::Positive))
    }
}

impl UnsignedAbs for IBig {
    type Output = UBig;

    #[inline]
    fn unsigned_abs(self) -> UBig {
        UBig(self.0.with_sign(Sign::Positive))
    }
}

impl UnsignedAbs for &IBig {
    type Output = UBig;

    #[inline]
    fn unsigned_abs(self) -> UBig {
        UBig(self.0.clone().with_sign(Sign::Positive))
    }
}

impl Mul<Sign> for UBig {
    type Output = IBig;

    #[inline]
    fn mul(self, rhs: Sign) -> Self::Output {
        IBig(self.0.with_sign(rhs))
    }
}

impl Mul<UBig> for Sign {
    type Output = IBig;

    #[inline]
    fn mul(self, rhs: UBig) -> Self::Output {
        IBig(rhs.0.with_sign(self))
    }
}

impl Mul<Sign> for IBig {
    type Output = IBig;

    #[inline]
    fn mul(self, rhs: Sign) -> Self::Output {
        let sign = self.sign() * rhs;
        IBig(self.0.with_sign(sign))
    }
}

impl Mul<IBig> for Sign {
    type Output = IBig;

    #[inline]
    fn mul(self, rhs: IBig) -> Self::Output {
        let sign = self * rhs.sign();
        IBig(rhs.0.with_sign(sign))
    }
}

impl MulAssign<Sign> for IBig {
    fn mul_assign(&mut self, rhs: Sign) {
        *self = core::mem::take(self) * rhs;
    }
}
