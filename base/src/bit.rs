//! Trait definitions for bitwise operations.
//!
//! Most traits are only implemented for unsigned integers yet.

// TODO: remove AndNot as it's not widely used, it's only used internally in UBig

/// Bitwise AND NOT operation.
///
/// `x.and_not(y)` is equivalent to `x & !y` for primitive integers.
///
/// # Examples
///
/// ```
/// use dashu_base::AndNot;
/// assert_eq!((0xff as u32).and_not(0x1111 as u32), 0xee);
/// ```
pub trait AndNot<Rhs = Self> {
    type Output;

    fn and_not(self, rhs: Rhs) -> Self::Output;
}

/// Common bit operations for integers
pub trait BitTest {
    /// Get the minimum required number of bits to represent this integer
    fn bit_len(&self) -> usize;

    /// Get the n-th bit of the integer
    fn bit(&self, n: usize) -> bool;

    /// Get the number of trailing zeros in the integer
    fn trailing_zeros(&self) -> Option<usize>;
}

/// Next power of two.
///
/// # Examples
/// ```
/// # use dashu_base::PowerOfTwo;
///
/// let n = 5u32;
/// assert!(!n.is_power_of_two());
/// assert_eq!(n.next_power_of_two(), 8);
/// ```
pub trait PowerOfTwo {
    fn is_power_of_two(&self) -> bool;
    fn next_power_of_two(self) -> Self;
}

macro_rules! impl_bit_ops_prim {
    ($($T:ty)*) => {$(
        impl AndNot for $T {
            type Output = $T;
            #[inline]
            fn and_not(self, rhs: $T) -> Self::Output {
                self & !rhs
            }
        }

        impl BitTest for $T {
            #[inline]
            fn bit_len(&self) -> usize {
                (<$T>::BITS - self.leading_zeros()) as usize
            }
            #[inline]
            fn bit(&self, position: usize) -> bool {
                self & (1 << position) > 0
            }
            #[inline]
            fn trailing_zeros(&self) -> Option<usize> {
                if *self == 0 {
                    None
                } else {
                    Some(<$T>::trailing_zeros(*self) as usize)
                }
            }
        }

        impl PowerOfTwo for $T {
            #[inline]
            fn is_power_of_two(&self) -> bool {
                <$T>::is_power_of_two(*self)
            }
            #[inline]
            fn next_power_of_two(self) -> $T {
                <$T>::next_power_of_two(self)
            }
        }
    )*}
}
impl_bit_ops_prim!(u8 u16 u32 u64 u128 usize);
