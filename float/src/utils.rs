use crate::ibig_ext::{log, magnitude};
use core::convert::TryInto;
use dashu_base::DivRem;
use dashu_int::IBig;

/// Get the integer k such that `radix^(k-1) <= value < radix^k`.
/// If value is 0, then `k = 0` is returned.
pub fn get_precision<const X: usize>(value: &IBig) -> usize {
    if value.is_zero() {
        return 0;
    };

    let e = log(&magnitude(value), X);
    let e: usize = e.try_into().unwrap();
    e + 1
}

/// "Left shifting" in given radix, i.e. multiply by a power of radix
#[inline]
pub fn shl_radix<const X: usize>(value: &mut IBig, exp: usize) {
    if exp != 0 {
        match X {
            2 => *value <<= exp,
            10 => {
                *value *= IBig::from(5).pow(exp);
                *value <<= exp;
            }
            16 => *value <<= 4 * exp,
            _ => *value *= IBig::from(X).pow(exp),
        }
    }
}

/// "Right shifting" in given radix, i.e. divide by a power of radix
#[inline]
pub fn shr_radix<const X: usize>(value: &mut IBig, exp: usize) {
    if exp != 0 {
        match X {
            2 => *value >>= exp,
            10 => {
                *value >>= exp;
                *value /= IBig::from(5).pow(exp);
            }
            16 => *value >>= 4 * exp,
            _ => *value /= IBig::from(X).pow(exp),
        }
    }
}

/// "Right shifting" in given radix, i.e. divide by a power of radix.
/// It returns the "shifted" value and the "remainder" part of integer that got removed
#[inline]
pub fn shr_rem_radix<const X: usize>(value: &IBig, exp: usize) -> (IBig, IBig) {
    if exp != 0 {
        match X {
            2 => {
                // FIXME: a dedicate method to extract low bits for IBig might be helpful here
                let rem = value & ((IBig::one() << exp) - 1);
                (value >> exp, rem)
            }
            10 => {
                let rem1 = value & ((IBig::one() << exp) - 1);
                let (q, rem2) = (value >> exp).div_rem(IBig::from(5).pow(exp));
                let rem = (rem2 << exp) + rem1;
                (q, rem)
            }
            16 => {
                let rem = value & ((IBig::one() << (4 * exp)) - 1);
                (value >> 4 * exp, rem)
            }
            _ => value.div_rem(IBig::from(X).pow(exp)),
        }
    } else {
        (value.clone(), IBig::zero())
    }
}

#[inline]
pub fn shr_rem_radix_in_place<const X: usize>(value: &mut IBig, exp: usize) -> IBig {
    if exp != 0 {
        match X {
            2 => {
                // FIXME: a dedicate method to extract low bits for IBig might be helpful here
                let rem = &*value & ((IBig::one() << exp) - 1);
                *value >>= exp;
                rem
            }
            10 => {
                let rem1 = &*value & ((IBig::one() << exp) - 1);
                let (q, rem2) = (&*value >> exp).div_rem(IBig::from(5).pow(exp));
                *value = q;
                let rem = (rem2 << exp) + rem1;
                rem
            }
            16 => {
                let rem = &*value & ((IBig::one() << (4 * exp)) - 1);
                *value >>= 4 * exp;
                rem
            }
            _ => {
                let (q, r) = (&*value).div_rem(IBig::from(X).pow(exp));
                *value = q;
                r
            }
        }
    } else {
        IBig::zero()
    }
}
