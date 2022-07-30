//! - Rounding is ensured in type level
//! - Precision is stored inside the numbers
//! - The higher precision will be used if two oprands have different precision
//! - Conversion from f32 and f64 is only implemented for BinaryRepr
//! - Conversion from and to str is limited to native radix. To print or parse with different
//!   radix, use FloatRepr::with_radix() to convert. (printing with certain radices is permitted,
//!   but need to specify explicitly, to print decimal numbers, one can use scientific representation
//!   or use the alternate flag)

#![cfg_attr(not(feature = "std"), no_std)]

// TODO: reference crates: twofloat, num-bigfloat, rust_decimal, bigdecimal, scientific
mod add;
mod cmp;
mod convert;
mod div;
mod fmt;
mod ibig_ext;
mod mul;
mod parse;
mod repr;
pub mod round;
mod sign;
mod utils;

pub use repr::FloatRepr;

/// Multi-precision float number with binary exponent and [Zero][round::mode::Zero] rounding mode
pub type FBig = FloatRepr<2, round::mode::Zero>;

/// Multi-precision decimal number with decimal exponent and [HalfAway][round::mode::HalfAway] rounding mode
pub type DBig = FloatRepr<10, round::mode::HalfAway>;
