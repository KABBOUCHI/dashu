use super::RootRem;
use crate::DivRem;

trait NormalizedRootRem : Sized {
    type OutputRoot;

    /// Square root with the normalized input such that highest or second
    /// highest bit are set. For internal use only.
    fn normed_sqrt_rem(self) -> (Self::OutputRoot, Self);

    /// Cubic root with the normalized input such that at least one of the
    /// highest three bits are set.  For internal use only.
    fn normed_cbrt_rem(self) -> (Self::OutputRoot, Self);
}

// Estimations of normalized 1/sqrt(x) with 9 bits precision. Specifically
// (rsqrt_tab[i] + 0x100) / 0x200 ≈ (sqrt(32) / sqrt(32 + i))
const RSQRT_TAB: [u8; 96] = [
    0xfc, 0xf4, 0xed, 0xe6, 0xdf, 0xd9, 0xd3, 0xcd, 0xc7, 0xc2, 0xbc, 0xb7,
    0xb2, 0xad, 0xa9, 0xa4, 0xa0, 0x9c, 0x98, 0x94, 0x90, 0x8c, 0x88, 0x85,
    0x81, 0x7e, 0x7b, 0x77, 0x74, 0x71, 0x6e, 0x6b, 0x69, 0x66, 0x63, 0x61,
    0x5e, 0x5b, 0x59, 0x57, 0x54, 0x52, 0x50, 0x4d, 0x4b, 0x49, 0x47, 0x45,
    0x43, 0x41, 0x3f, 0x3d, 0x3b, 0x39, 0x37, 0x36, 0x34, 0x32, 0x30, 0x2f,
    0x2d, 0x2c, 0x2a, 0x28, 0x27, 0x25, 0x24, 0x22, 0x21, 0x1f, 0x1e, 0x1d,
    0x1b, 0x1a, 0x19, 0x17, 0x16, 0x15, 0x14, 0x12, 0x11, 0x10, 0x0f, 0x0d,
    0x0c, 0x0b, 0x0a, 0x09, 0x08, 0x07, 0x06, 0x05, 0x04, 0x03, 0x02, 0x01,
];

// Estimations of normalized 1/cbrt(x) with 9 bits precision. Specifically
// (rcbrt_tab[i] + 0x100) / 0x200 ≈ (cbrt(8) / cbrt(8 + i))
const RCBRT_TAB: [u8; 56] = [
    0xf6, 0xe4, 0xd4, 0xc6, 0xb9, 0xae, 0xa4, 0x9b,
    0x92, 0x8a, 0x83, 0x7c, 0x76, 0x70, 0x6b, 0x66,
    0x61, 0x5c, 0x57, 0x53, 0x4f, 0x4b, 0x48, 0x44,
    0x41, 0x3e, 0x3b, 0x38, 0x35, 0x32, 0x2f, 0x2d,
    0x2a, 0x28, 0x25, 0x23, 0x21, 0x1f, 0x1d, 0x1b,
    0x19, 0x17, 0x15, 0x13, 0x11, 0x10, 0x0e, 0x0c,
    0x0b, 0x09, 0x08, 0x06, 0x05, 0x03, 0x02, 0x01
];

// util: high part of 32bit widening mul
#[inline]
fn wmul32_hi(a: u32, b: u32) -> u32 {
    ((a as u64) * (b as u64) >> 32) as u32
}

impl NormalizedRootRem for u64 {
    type OutputRoot = u32;

    fn normed_sqrt_rem(self) -> (u32, u64) {
        // Use newton's method on 1/sqrt(n)
        // x_{i+1} = x_i * (3 - n*x_i^2) / 2
        debug_assert!(self.leading_zeros() <= 1);

        // step1: lookup initial estimation of normalized 1/√n. The lookup table uses the highest 7 bits,
        // since the input is normalized, the lookup index must be larger than 2**(7-2) = 32.
        // then the retrieved r ≈ √32 / √(n >> 57) * 0x200 = 1 / √(n >> 62) / 2^9 = 2^40 / √n.
        // the sqrt(32) in the nominator (effectively √2) will be eliminated by the odd shifting of n.
        let n32 = (self >> 32) as u32;
        let r = 0x100 | RSQRT_TAB[(n32 >> 25) as usize - 32] as u32; // 9 bits
        
        // step2: first Newton iteration (without dividing by 2)
        // r will be an estimation of 2^(40+22) / √n with 16 bits effective precision
        let r = ((3 * r) << 21) - wmul32_hi(n32, (r * r * r) << 5); // 31 bits

        // step3: second Newton iteration (without dividing by 2)
        // r will be an estimation of normalized 2^(40+19) / √n with 32 bits effective precision
        let t = (3 << 28) - wmul32_hi(r, wmul32_hi(r, n32));
        let r = wmul32_hi(r, t); // 28 bits

        // step4: √n = x * 1/√n
        let r = r << 4;
        let mut s = wmul32_hi(r, n32) * 2;
        s -= 10; // to make sure it's an underestimate

        // step5: third Newton iteration on √n
        let e = self - (s as u64) * (s as u64);
        s += wmul32_hi((e >> 32) as u32, r);

        // step6: fix the estimation error, at most 2 steps are needed
        // if we use more bits to estimate the initial guess, less steps can be required
        let mut e = self - (s as u64) * (s as u64);
        let mut elim = 2 * s as u64 + 1;
        while e >= elim {
            s += 1;
            e -= elim;
            elim += 2;
        }

        (s, e)
    }

    // note that the input should be normalized to 63 bits instead of 64
    fn normed_cbrt_rem(self) -> (u32, u64) {
        // Use newton's method on 1/sqrt(n)
        // x_{i+1} = x_i * (4 - n*x_i^3) / 3
        debug_assert!(self.leading_zeros() >= 1 && self.leading_zeros() <= 4);

        // step2: lookup initial estimation of 1/∛x. The lookup table uses the highest 6 bits
        // retrieved r ≈ ∛8 / ∛(n >> 57) * 0x200 = 1 / ∛(n >> 60) * 2^9 = 2^29 / ∛n.
        let n32 = (self >> 32) as u32;
        let r = 0x100 | RCBRT_TAB[(n32 >> 25) as usize - 8] as u32; // 9bit int
        
        // step3: first Newton iteration
        // r = 2^52 / ∛n
        let t = (4 << 23) - wmul32_hi(n32, r * r * r);
        let r = r * (t / 3); // 32bit

        // step4: second Newton iteration
        // r = 2^48 / ∛n
        let t = (4 << 28) - wmul32_hi(r, wmul32_hi(r, wmul32_hi(r, n32)));
        let r = wmul32_hi(r, t) / 3; // 28bit

        // step5: ∛x = x * (1/∛x)^2
        let r = r - 1; // to make sure s is underestimate
        let mut s = wmul32_hi(r, wmul32_hi(r, n32));

        // step6: fix the estimation error, at most 2 steps are needed
        // if we use more bits to estimate the initial guess, less steps can be required
        let s2 = (s as u64) * (s as u64);
        let mut e = self - s2 * (s as u64);
        let mut elim = 3 * (s2 + s as u64) + 1;
        while e >= elim {
            s += 1;
            e -= elim;
            elim += 6 * (s as u64);
        }

        (s, e)
    }
}

impl NormalizedRootRem for u128 {
    type OutputRoot = u64;

    fn normed_sqrt_rem(self) -> (u64, u128) {
        debug_assert!(self.leading_zeros() <= 1);
        const HALF_BITS: u32 = u64::BITS / 2;

        // the following algorithm is based on "Karatsuba Square Root":
        // assume n = a3*b^3 + a2*b^2 + a1*b + a0, b=2^k
        // 1. calculate sqrt on high part:
        //     s1, r1 = sqrt_rem(a3*b + a2);
        // 2. estimate the root with low part
        //     q, u = div_rem(r1*b + a1, 2*s1)
        //     s = s1 * b + q
        //     r = u*b + a0 - q^2
        // 3. if a3 is normalized, then s is either correct or 1 too big.
        //    r is negative in the latter case, needs adjustment
        //     if r < 0 {
        //         r += 2*s - 1
        //         s -= 1
        //     }
        //
        
        // step1: calculate sqrt on high parts
        let (n0, n1) = (self & u64::MAX as u128, self >> u64::BITS);
        let (n0, n1) = (n0 as u64, n1 as u64);
        let (s1, r1) = n1.normed_sqrt_rem();

        // step2: estimate the result with low parts
        // note that r1 <= 2*s1 < 2^(HALF_BITS + 1)
        let r0 = r1 << (HALF_BITS - 1) | n0 >> (HALF_BITS + 1);
        let (mut q, mut u) = r0.div_rem(s1 as u64);
        if q >> HALF_BITS > 0 {
            // if q = 2^HALF_BITS, reduce the overestimate
            q -= 1;
            u += s1 as u64;
        }
        let mut s = (s1 as u64) << HALF_BITS | q;
        let r = (u << (HALF_BITS + 1)) + (n0 & (1 << (HALF_BITS + 1)) - 1);
        let q2 = q * q;
        let mut cc = (u >> (HALF_BITS - 1)) as i8 - (r < q2) as i8; // carry bit
        let mut r = r.wrapping_sub(q2);

        // step3: adjustment
        if cc < 0 {
            r = r.wrapping_add(s);
            cc += (r < s) as i8;
            s -= 1;
            r = r.wrapping_add(s);
            cc += (r < s) as i8;
        }
        (s, (cc as u128) << u64::BITS | r as u128)
    }

    fn normed_cbrt_rem(self) -> (u64, u128) {
        unimplemented!()
    }
}

impl RootRem for u64 {
    type Output = u64;

    #[inline]
    fn sqrt_rem(self) -> (u64, u64) {
        if self == 0 {
            return (0, 0);
        }

        // normalize the input and call the normalized subroutine
        let shift = self.leading_zeros() & (u32::MAX - 1); // make sure shift is divisible by 2
        let (root, mut rem) = (self << shift).normed_sqrt_rem();
        let root = (root >> (shift / 2)) as u64;
        if shift != 0 {
            rem = self - root * root;
        }
        (root, rem)
    }

    fn cbrt_rem(self) -> (u64, u64) {
        if self == 0 {
            return (0, 0);
        }

        // the precomputed table only supports integer up to 63 bits, use ∛2 to fix 1 bit error
        if self.leading_zeros() == 0 {
            const CBRT2: u64 = 1385297844439; // 40bit estimation of floor(∛2), error < 2^-42
            let (s, _) = Self::cbrt_rem(self >> 1); // s.bit_len() <= 20
            let mut s = s * CBRT2 >> 40;

            // fix the estimation, at most 2 steps are needed
            let s2 = s * s;
            let mut e = self - s2 * s;
            let mut elim = 3 * (s2 + s) + 1;
            while e >= elim {
                s += 1;
                e -= elim;
                elim += 6 * (s as u64);
            }

            return (s, e)
        }

        // normalize the input and call the normalized subroutine
        let mut shift = self.leading_zeros() - 1;
        shift -= shift % 3; // align to 63 bits
        let (root, mut rem) = (self << shift).normed_cbrt_rem();
        let root = (root >> (shift / 3)) as u64;
        if shift != 0 {
            rem = self - root * root * root;
        }
        (root as u64, rem)
    }

    #[inline]
    fn nth_root_rem(self, _n: usize) -> (u64, u64) {
        unimplemented!()
    }
}

impl RootRem for u128 {
    type Output = u128;

    #[inline]
    fn sqrt_rem(self) -> (u128, u128) {
        if self == 0 {
            return (0, 0);
        }
        if self <= u64::MAX as u128 {
            let (s, r) = (self as u64).sqrt_rem();
            return (s as u128, r as u128);
        }

        // normalize the input and call the normalized subroutine
        let shift = self.leading_zeros() & (u32::MAX - 1); // make sure shift is divisible by 2
        let (root, mut rem) = (self << shift).normed_sqrt_rem();
        let root = (root >> (shift / 2)) as u128;
        if shift != 0 {
            rem = self - root * root;
        }
        (root, rem)
    }

    #[inline]
    fn cbrt_rem(self) -> (u128, u128) {
        if self == 0 {
            return (0, 0);
        }

        unimplemented!()
    }
    
    #[inline]
    fn nth_root_rem(self, _n: usize) -> (u128, u128) {
        unimplemented!()
    }
}

// TODO: forward sqrt to f64 if std enabled, don't forward cbrt

#[cfg(test)]
mod tests {
    use super::*;
    use rand::random;

    #[test]
    fn test_sqrt() {
        macro_rules! random_case {
            ($T:ty) => {
                let n: $T = random();
                let (root, rem) = n.sqrt_rem();
                assert!(rem <= root * 2, "sqrt({}) remainder too large", n);
                assert_eq!(n, root * root + rem, "sqrt({}) != {}, {}", n, root, rem);
            }
        }

        const N: u32 = 10000;
        for _ in 0..N {
            random_case!(u64);
            random_case!(u128);
        }
    }

    #[test]
    fn test_cbrt() {
        const N: u32 = 10000;
        for _ in 0..N {
            let n: u64 = random();
            let (root, rem) = n.cbrt_rem();
            assert!(rem <= 3 * root * (root + 1));
            assert_eq!(n, root * root * root + rem, "cbrt({}) != {}, {}", n, root, rem);
        }
    }
}