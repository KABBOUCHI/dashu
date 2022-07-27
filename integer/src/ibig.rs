//! Signed big integer.

use crate::{
    repr::{Repr, TypedRepr, TypedReprRef},
    sign::Sign,
    UBig,
};

/// Signed big integer.
///
/// Arbitrarily large signed integer.
///
/// # Examples
///
/// ```
/// # use dashu_int::{error::ParseError, IBig};
/// let a = IBig::from(408580953453092208335085386466371u128);
/// let b = IBig::from(-0x1231abcd4134i64);
/// let c = IBig::from_str_radix("a2a123bbb127779cccc123", 32)?;
/// let d = IBig::from_str_radix("-1231abcd4134", 16)?;
/// assert_eq!(a, c);
/// assert_eq!(b, d);
/// # Ok::<(), ParseError>(())
/// ```
#[derive(Eq, Hash, PartialEq)]
#[repr(transparent)]
pub struct IBig(pub(crate) Repr);

impl IBig {
    #[inline]
    pub(crate) fn as_sign_repr(&self) -> (Sign, TypedReprRef<'_>) {
        self.0.as_sign_typed()
    }

    #[inline]
    pub(crate) fn into_sign_repr(self) -> (Sign, TypedRepr) {
        self.0.into_sign_typed()
    }

    /// Get the sign of the [IBig]. Zero value has a positive sign.
    /// 
    /// # Examples
    /// 
    /// ```
    /// # use dashu_int::{IBig, Sign};
    /// assert_eq!(IBig::zero().sign(), Sign::Positive);
    /// assert_eq!(IBig::from(2).sign(), Sign::Positive);
    /// assert_eq!(IBig::from(-3).sign(), Sign::Negative);
    /// ```
    #[inline]
    pub fn sign(&self) -> Sign {
        self.0.sign()
    }

    /// Convert the [IBig] into its [Sign] and [UBig] magnitude
    /// 
    /// # Examples
    /// 
    /// ```
    /// # use dashu_int::{IBig, Sign, UBig};
    /// assert_eq!(IBig::zero().into_parts(), (Sign::Positive, UBig::zero()));
    /// assert_eq!(IBig::one().into_parts(), (Sign::Positive, UBig::one()));
    /// assert_eq!(IBig::neg_one().into_parts(), (Sign::Negative, UBig::one()));
    /// ```
    #[inline]
    pub fn into_parts(self) -> (Sign, UBig) {
        let sign = self.0.sign();
        let mag = self.0.with_sign(Sign::Positive);
        (sign, UBig(mag))
    }

    
    /// Create an [IBig] from the [Sign] and [UBig] magnitude
    /// 
    /// # Examples
    /// 
    /// ```
    /// # use dashu_int::{IBig, Sign, UBig};
    /// assert_eq!(IBig::from_parts(Sign::Positive, UBig::zero()), IBig::zero());
    /// assert_eq!(IBig::from_parts(Sign::Positive, UBig::one()), IBig::one());
    /// assert_eq!(IBig::from_parts(Sign::Negative, UBig::one()), IBig::neg_one());
    /// ```
    #[inline]
    pub fn from_parts(sign: Sign, magnitude: UBig) -> Self {
        IBig(magnitude.0.with_sign(sign))
    }

    /// Create an IBig with value 0
    #[inline]
    pub const fn zero() -> Self {
        IBig(Repr::zero())
    }

    /// Check whether the value of IBig is 0
    #[inline]
    pub const fn is_zero(&self) -> bool {
        self.0.is_zero()
    }

    /// Create an IBig with value 1
    #[inline]
    pub const fn one() -> Self {
        IBig(Repr::one())
    }

    /// Check whether the value of IBig is 1
    #[inline]
    pub const fn is_one(&self) -> bool {
        self.0.is_one()
    }

    /// Create an IBig with value -1
    #[inline]
    pub const fn neg_one() -> IBig {
        IBig(Repr::neg_one())
    }
}

// This custom implementation is necessary due to https://github.com/rust-lang/rust/issues/98374
impl Clone for IBig {
    #[inline]
    fn clone(&self) -> IBig {
        IBig(self.0.clone())
    }

    #[inline]
    fn clone_from(&mut self, source: &IBig) {
        self.0.clone_from(&source.0)
    }
}
