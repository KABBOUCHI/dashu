use crate::{
    repr::{Context, Repr, Word},
    round::{mode, Round},
};
use core::marker::PhantomData;
use dashu_base::Sign;
use dashu_int::{DoubleWord, IBig};

/// An arbitrary precision floating number represented as `signficand * base^exponent`, with a precision
/// such that `|signficand| < base^precision`. The representation is always normalized (nonzero signficand
/// is not divisible by base, or zero signficand with zero exponent). But the precision limit is not always
/// enforced. In rare cases, the significand can have one more digit than the precision limit.
///
/// The rounding mode of operations between the float numbers is defined by `Rounding`, its value has to
/// be one of [RoundingMode]. Operations are permitted only between float numbers with the same base and
/// rounding mode. Note that the rounding is only for operations, it's not "associated" with the value.
/// For example, for a `correct` subtraction, the two operands should have reverse rounding direction, but
/// the rounding mode of [FBig] only determines the rounding direction of this subtraction operation.
///
/// # Generic Parameters
/// The const generic parameters will be abbreviated as BASE -> B, Rounding -> R.
/// BASE should be in range \[2, isize::MAX\], and Rounding value has to be one of [RoundingMode]
///
/// # Infinity
///
/// This struct supports representation the infinity, but the infinity is only supposed to be used as sentinels.
/// That is, only equality test and comparison are implemented for infinity. Any other operations performed
/// with infinity will lead to panic.
///
/// The infinities are represented as:
/// * [Positive infinity][FloatRepr::INFINITY] (+∞): signficand = 0, exponent > 0
/// * [Negative infinity][FloatRepr::NEG_INFINITY] (-∞): signficand = 0, exponenet < 0
///
pub struct FBig<RoundingMode: Round = mode::Zero, const BASE: Word = 2> {
    pub(crate) repr: Repr<BASE>,
    pub(crate) context: Context<RoundingMode>,
}

impl<R: Round, const B: Word> FBig<R, B> {
    /// Create a [FBig] instance from raw parts, internal use only
    #[inline]
    pub(crate) const fn new(repr: Repr<B>, context: Context<R>) -> Self {
        Self { repr, context }
    }

    const fn zero() -> Self {
        Self::new(Repr::zero(), Context::new(1))
    }
    /// [FBig] with value 0 and precision 1
    pub const ZERO: Self = Self::zero();

    const fn one() -> Self {
        Self::new(Repr::one(), Context::new(1))
    }
    /// [FBig] with value 1 and precision 1
    pub const ONE: Self = Self::one();

    const fn neg_one() -> Self {
        Self::new(Repr::neg_one(), Context::new(1))
    }
    /// [FBig] with value -1 and precision 1
    pub const NEG_ONE: Self = Self::neg_one();

    const fn inf() -> Self {
        Self::new(Repr::infinity(), Context::new(0))
    }
    pub const INFINITY: Self = Self::inf();

    const fn neg_inf() -> Self {
        Self::new(Repr::neg_infinity(), Context::new(0))
    }
    pub const NEG_INFINITY: Self = Self::neg_inf();

    #[inline]
    pub const fn sign(&self) -> Sign {
        if self.repr.significand.is_zero() {
            if self.repr.exponent >= 0 {
                Sign::Positive
            } else {
                Sign::Negative
            }
        } else {
            self.repr.significand.sign()
        }
    }

    /// Get the maximum precision set for the float number.
    #[inline]
    pub const fn precision(&self) -> usize {
        self.context.precision
    }

    /// Get the actual precision needed for the float number.
    ///
    /// This is also the actual precision needed for the float number. Shrink to this value using [Self::with_precision] will not cause loss of float precision.
    #[inline]
    pub fn digits(&self) -> usize {
        self.repr.digits()
    }

    /// Get the context associated with the float number
    #[inline]
    pub const fn context(&self) -> Context<R> {
        self.context
    }

    /// Convert raw parts into a float number, the precision will be inferred from significand
    /// (the lowest k such that `significand < radix^k`)
    ///
    /// # Panics
    ///
    /// Panics if the significand is larger than `radix^usize::MAX`
    #[inline]
    pub fn from_parts(significand: IBig, exponent: isize) -> Self {
        // TODO: check we are not using this function internally because we enforce normalized representation
        let repr = Repr::new(significand, exponent);
        let precision = repr.digits().max(1); // set precision to 1 if signficand is zero
        Self {
            repr,
            context: Context {
                precision,
                _marker: PhantomData,
            },
        }
    }

    #[inline]
    pub const fn from_parts_const(
        sign: Sign,
        mut significand: DoubleWord,
        mut exponent: isize,
    ) -> Self {
        if significand == 0 {
            return Self::ZERO;
        }

        let mut digits = 0;

        // normalize
        if B.is_power_of_two() {
            let base_bits = B.trailing_zeros();
            let shift = significand.trailing_zeros() / base_bits;
            significand >>= shift * base_bits;
            exponent += shift as isize;
            digits = ((DoubleWord::BITS - significand.leading_zeros() + base_bits - 1) / base_bits)
                as usize;
        } else {
            let mut pow: DoubleWord = 1;
            while significand % (B as DoubleWord) == 0 {
                significand /= B as DoubleWord;
                exponent += 1;
            }
            while let Some(next) = pow.checked_mul(B as DoubleWord) {
                digits += 1;
                if next > significand {
                    break;
                }
                pow = next;
            }
        }

        let repr = Repr {
            significand: IBig::from_parts_const(sign, significand),
            exponent,
        };
        Self {
            repr,
            context: Context::new(digits),
        }
    }

    /// Convert the float number into raw (signficand, exponent) parts
    #[inline]
    pub fn into_parts(self) -> (IBig, isize) {
        (self.repr.significand, self.repr.exponent)
    }

    fn ulp(&self) -> Self {
        // reference: https://docs.python.org/3/library/math.html#math.ulp
        unimplemented!()
    }
    fn ceil(&self) -> IBig {
        unimplemented!()
    }
    fn floor(&self) -> IBig {
        unimplemented!()
    }
    fn trunc(&self) -> Self {
        unimplemented!()
    }
    fn fract(&self) -> Self {
        unimplemented!()
    }
}

// This custom implementation is necessary due to https://github.com/rust-lang/rust/issues/98374
impl<R: Round, const B: Word> Clone for FBig<R, B> {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            repr: self.repr.clone(),
            context: self.context.clone(),
        }
    }

    #[inline]
    fn clone_from(&mut self, source: &Self) {
        self.repr.clone_from(&source.repr);
        self.context = source.context;
    }
}

impl<R: Round, const B: Word> Default for FBig<R, B> {
    /// Default value: 0.
    #[inline]
    fn default() -> Self {
        Self::ZERO
    }
}
