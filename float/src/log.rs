
use dashu_base::EstimatedLog2;
use dashu_int::IBig;

use crate::{
    fbig::FBig,
    repr::{Context, Word},
    round::{Round, Rounded}
};

impl<const B: Word, R: Round> EstimatedLog2 for FBig<B, R> {
    // currently a Word has at most 64 bits, so log2() < f32::MAX
    fn log2_bounds(&self) -> (f32, f32) {
        // log(s*B^e) = log(s) + e*log(B)
        let (logs_lb, logs_ub) = self.repr.significand.log2_bounds();
        let (logb_lb, logb_ub) = if B.is_power_of_two() {
            let log = B.trailing_zeros() as f32;
            (log, log)
        } else {
            B.log2_bounds()
        };
        let e = self.repr.exponent as f32;
        if self.repr.exponent >= 0 {
            (logs_lb + e * logb_lb, logs_ub + e * logb_ub)
        } else {
            (logs_lb + e * logb_ub, logs_ub + e * logb_lb)
        }
    }
}

impl<const B: Word, R: Round> FBig<B, R> {
    #[inline]
    pub fn ln(&self) -> Self {
        self.context.ln(self).value()
    }
}

impl<R: Round> Context<R> {
    /// Calculate log(2)
    /// 
    /// The precision of the output will be larger than self.precision
    #[inline]
    fn ln2<const B: Word>(&self) -> FBig<B, R> {
        // log(2) = 4L(6) + 2L(99)
        // see formula (24) from Gourdon, Xavier, and Pascal Sebah.
        // "The Logarithmic Constant: Log 2." (2004)
        4 * self.iacoth(6.into()) + 2 * self.iacoth(99.into())
    }

    /// Calculate log(2)
    /// 
    /// The precision of the output will be larger than self.precision
    #[inline]
    fn ln10<const B: Word>(&self) -> FBig<B, R> {
        // log(10) = log(2) + log(5) = 3log(2) + 2L(9)
        // see example (17) from "The Logarithmic Constant: Log 2"
        3 * self.ln2() + 2 * self.iacoth(9.into())
    }

    /// Calculate L(n) = acoth(n) = atanh(1/n) = 1/2 log((n+1)/(n-1))
    /// 
    /// This method is intended to be used in logarithm calculation,
    /// so the precision of the output will be larger than desired precision.
    fn iacoth<const B: Word>(&self, n: IBig) -> FBig<B, R> {
        /* 
         * use Maclaurin series:
         *       1    1     n+1             1
         * atanh(—) = — log(———) =  Σ  ———————————
         *       n    2     n-1    i≥0 n²ⁱ⁺¹(2i+1)
         * 
         * Therefore to achieve precision B^p, the series should be stopped at
         *    n²ⁱ⁺¹(2i+1) / n >= B^p
         * => 2i*ln(n) + ln(2i+1) >= p ln(B)
         * => 2i*ln(n) >= p ln(B)
         * => 2i >= p/log_B(n)
         * let k = 2i + 1, we choose max{k} = p/log_B(n) + 1
         * 
         * There will be i summations when calculating the series, to prevent
         * loss of significant, we needs log_B(i) guard digits.
         *    log_B[(p/log_B(n) - 1) / 2]
         * <= log_B(p/2log_B(n))
         *  = log_B(p/2) - log_B(log_B(n))
         * <= log_B(p/2)
         */
        let max_k = (self.precision as f32 * B.log2_bounds().1 / n.log2_bounds().0) as usize;
        let guard_digits = ((self.precision / 2).log2_bounds().1 / B.log2_bounds().1) as usize;
        let (max_k, guard_digits) = (max_k + 2, guard_digits + 2); // add extras to ensure precise result
        let work_context = Self::new(self.precision + guard_digits);

        let n = work_context.convert_int(n);
        let inv = FBig::ONE / n;
        let inv2 = inv.square();
        let mut sum = inv.clone();
        let mut pow = inv;

        for k in (3..=max_k).step_by(2) {
            pow *= &inv2;
            sum += &pow / work_context.convert_int::<B>(k.into());
        }
        sum
    }

    /// Calculate the natural logarithm of the number x
    pub fn ln<const B: Word>(&self, x: &FBig<B, R>) -> Rounded<FBig<B, R>> {
        // Simple algorithm:
        // log(x) = log(x/2^s) + slog2
        // such that x*2^s is close to but larger than 1,
        // so s = -floor(log2(x))
        let x = x.clone().with_precision(self.precision + 1).value();
        let log2 = x.log2_bounds().0;
        let x_scaled = if log2 >= 0. {
            let pow = IBig::ONE << log2 as usize;
            x / pow
        } else {
            let pow = IBig::ONE << (-log2) as usize;
            x * pow
        };
        // TODO: assert x_scaled > 1

        // after the number is scaled to nearly one, use Maclaurin series on log(x) = 2atanh(z)
        // let z = (x-1)/(x+1) < 1, log(x) = 2atanh(z) = 2Σ(zⁱ/i) for i = 1,3,5,...
        // Similar to iacoth, the required iterations stop at i = -p/log_B(z) + 1,
        // and we need log_B(p/2) guard bits
        let z = (&x_scaled - FBig::ONE) / (x_scaled + FBig::ONE);
        let max_k = (self.precision as f32 * B.log2_bounds().1 / -z.log2_bounds().0) as usize;
        let guard_digits = ((self.precision / 2).log2_bounds().1 / B.log2_bounds().1) as usize;
        let (max_k, guard_digits) = (max_k + 2, guard_digits + 2); // add extras to ensure precise result
        let work_context = Self::new(self.precision + guard_digits);

        let z2 = z.square();
        let mut pow = z.clone();
        let mut sum = z.clone();

        for k in (3..=max_k).step_by(2) {
            pow *= &z2;
            sum += &pow / work_context.convert_int::<B>(k.into());
        }

        // compose the logarithm of the original number
        let result = if log2 >= 0. {
            2 * sum + self.ln2() * IBig::from(log2 as usize)
        } else {
            2 * sum - self.ln2() * IBig::from((-log2) as usize)
        };
        result.with_precision(self.precision)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::round::mode;

    #[test]
    fn test_iacoth() {
        let context = Context::<mode::Zero>::new(10);
        let binary_6 = context.iacoth::<2>(6.into()).with_precision(10).value();
        assert_eq!(binary_6.repr.significand, 689);
        let decimal_6 = context.iacoth::<10>(6.into()).with_precision(10).value();
        assert_eq!(decimal_6.repr.significand, 1682361183);

        let context = Context::<mode::Zero>::new(40);
        let decimal_6 = context.iacoth::<10>(6.into()).with_precision(40).value();
        assert_eq!(decimal_6.repr.significand, IBig::from_str_radix("1682361183106064652522967051084960450557", 10).unwrap());

        let context = Context::<mode::Zero>::new(201);
        let binary_6 = context.iacoth::<2>(6.into()).with_precision(201).value();
        assert_eq!(binary_6.repr.significand, IBig::from_str_radix("2162760151454160450909229890833066944953539957685348083415205", 10).unwrap());
    }

    #[test]
    fn test_ln2_ln10() {
        let context = Context::<mode::Zero>::new(45);
        let decimal_ln2 = context.ln2::<10>().with_precision(45).value();
        assert_eq!(decimal_ln2.repr.significand, IBig::from_str_radix("693147180559945309417232121458176568075500134", 10).unwrap());
        let decimal_ln10 = context.ln10::<10>().with_precision(45).value();
        assert_eq!(decimal_ln10.repr.significand, IBig::from_str_radix("230258509299404568401799145468436420760110148", 10).unwrap());

        let context = Context::<mode::Zero>::new(180);
        let binary_ln2 = context.ln2::<2>().with_precision(180).value();
        assert_eq!(binary_ln2.repr.significand, IBig::from_str_radix("1062244963371879310175186301324412638028404515790072203", 10).unwrap());
        let binary_ln10 = context.ln10::<2>().with_precision(180).value();
        assert_eq!(binary_ln10.repr.significand, IBig::from_str_radix("882175346869410758689845931257775553286341791676474847", 10).unwrap());
    }
}
