use alloc::alloc::Layout;
use dashu_base::{RootRem, DivRem};
use crate::{
    arch::word::{Word, DoubleWord},
    memory::{self, Memory}, primitive::{highest_dword, WORD_BITS, split_dword, double_word, extend_word},
    div, add::{add_in_place, sub_in_place, add_word_in_place, sub_one_in_place}, fast_div::FastDivideNormalized2,
    shift::shr_in_place_with_carry, sqr, mul::add_mul_word_in_place,
};

// n is the size of the output, or half the size of the input
pub fn memory_requirement_sqrt_rem(n: usize) -> Layout {
    if n == 2 {
        memory::zero_layout()
    } else {
        memory::max_layout(
            sqr::memory_requirement_exact(n),
            div::memory_requirement_exact(n, n - n/2)
        )
    }
}

// Requires a is normalized to 2n words (length must be even)
// Returns the carry of the remainder
pub fn sqrt_rem<'a>(b: &mut [Word], a: &mut [Word], memory: &mut Memory) -> bool {
    debug_assert!(a.len() % 2 == 0);
    debug_assert!(a.len() >= 4, "use native sqrt when a has less than 2 words");
    debug_assert!(a.len() == b.len() * 2);

    // shortcut when a has exactly 4 words
    if a.len() == 4 {
        return sqrt_rem_42(b, a);
    }

    /* 
     * the "Karatsuba Square Root" algorithm:
     * assume n = a*B^2 + b1*B + b0, B=2^k, a has 2k bits and
     * is normalized (the top two bits of a are not all zeros)
     * 1. calculate sqrt on high part:
     *     s1, r1 = sqrt_rem(a) (r1 <= 2*s1)
     * 2. estimate the root with low part
     *     q, u = div_rem(r1*B + b1, 2*s1)
     *     s = s1*B + q
     *     r = u*B + b0 - q^2
     *    at this step, since a is normalized, we have s1 >= B/2,
     *    therefore q <= floor((r1*B + b1) / B) <= r1 <= 2*s1
     *    also notice b1 < B <= 2*s1, so q <= B
     * 
     * 3. if a3 is normalized, then s is either correct or 1 too big.
     *    r is negative in the latter case, needs adjustment
     *     if r < 0 {
     *         r += 2*s - 1
     *         s -= 1
     *     }
     * 
     * Reference: Zimmermann, P. (1999). Karatsuba square root (Doctoral dissertation, INRIA).
     * https://hal.inria.fr/inria-00072854/en/
     */

    let n = a.len() / 2; // the length of a
    let split = n / 2; // the length of b0

    // step1: sqrt on the higher half
    // afterwards, s1 = b[split..], r1 = a[2*split..split + n]
    let r1_top = sqrt_rem(&mut b[split..], &mut a[2*split..], memory);
    if r1_top {
        // if the remainder `r1` has a carry, subtract `s1` from it so that the carry is removed
        let carry = sub_in_place(&mut a[2*split..split + n], &b[split..]);
        debug_assert!(carry);
    }

    // step2: estimate the result with lower half
    let fast_div_top = FastDivideNormalized2::new(highest_dword(b));
    let carry = div::div_rem_in_place(&mut a[split..split + n], &b[split..], fast_div_top, memory);
    b[..split].copy_from_slice(&a[n..split + n]);
    // by now q = b[..split], u = a[split..n], carry is true only if r1 >= s1.
    // also notice that r1 <= 2 * s1, if r1 was subtracted by s1, then r1 <= s1.
    // so r_top and carry are both true only if r1 == 2 * s1 at the beginning.
    // the top bit of q is true if either r_top or carry is true, but not both
    let _ = shr_in_place_with_carry(&mut b[..split], 1, (r1_top ^ carry) as _);
    let q_top = r1_top && carry; // this is true only when q = B, so b[..split] = 0

    let mut c = 0i8; // stores final carry of the remainder
    if a[split] & 1 != 0 {
        // this step fixs the error in u caused by using s1 as divisor instead of 2*s1
        c = add_in_place(&mut a[split..n], &b[split..]) as i8;
    }

    // store q^2 in high part of a, ignoring q_top.
    // afterwards, the q_top flag will be considered in the subtraction,
    // and the remaining error in q^2 will be fixed by step 3
    let (a_lo, a_hi) = a.split_at_mut(n);
    a_hi.fill(0);
    sqr::square(&mut a_hi[..2 * split], &b[..split], memory);
    if 2 * split < n {
        a_hi[2 * split] = q_top as Word;
    } else {
        c -= q_top as i8;
    }
    c -= sub_in_place(a_lo, a_hi) as i8;

    // step3: fix the estimation error if necessary
    if c < 0 {
        // r += 2*s - 1; s -= 1;
        // apply the q_top to s first, and then adjust s and r
        let overflow = add_word_in_place(&mut b[split..], q_top as _);
        c += add_mul_word_in_place(&mut a_lo[..split], 2, &b[..split]) as i8;
        c += 2 * overflow as i8;
        c -= sub_one_in_place(a_lo) as i8;
        let borrow = sub_one_in_place(&mut b[..n]);
        debug_assert!(!(overflow ^ borrow)); // borrow should happen if and only if when overflow is true
    }

    return c > 0;
}

// Special case when a has exactly 4 Words
fn sqrt_rem_42<'a>(b: &mut [Word], a: &mut [Word]) -> bool {
    debug_assert!(a.len() == 4 && b.len() == 2);

    // step1: sqrt on the higher half
    let (s1, r1) = highest_dword(a).sqrt_rem();
    let s1 = s1 as Word;

    // step2: estimate the result with lower half
    // here r0 = (r1*B + b1) / 2
    let (r1_lo, r1_hi) = split_dword(r1);
    let r0_hi = r1_hi << (WORD_BITS - 1) | r1_lo >> 1;
    let r0_lo = r1_lo << (WORD_BITS - 1) | a[1] >> 1;
    let (mut q, mut u) = double_word(r0_lo, r0_hi).div_rem(s1 as DoubleWord);
    if q >> WORD_BITS > 0 {
        // if q >= B (then q = B), reduce the overestimate
        q -= 1;
        u += s1 as DoubleWord;
    }
    u = u << 1 | (a[1] & 1) as DoubleWord;

    let q = q as Word; // now q must fit in a Word
    let (u_lo, u_hi) = split_dword(u);
    let mut s = double_word(q, s1);
    let q2 = extend_word(q) * extend_word(q);
    let (mut r, borrow) = double_word(a[0], u_lo).overflowing_sub(q2);
    let mut c: i8 = u_hi as i8 - borrow as i8;

    // step3: fix the estimation error if necessary
    if c < 0 {
        let (new_r, c1) = r.overflowing_add(s);
        s -= 1;
        let (new_r, c2) = new_r.overflowing_add(s);
        c += c1 as i8 + c2 as i8;
        r = new_r;
    }

    let (r_lo, r_hi) = split_dword(r);
    let (s_lo, s_hi) = split_dword(s);
    a[0] = r_lo; a[1] = r_hi;
    b[0] = s_lo; b[1] = s_hi;
    c > 0
}

#[cfg(test)]
mod tests {
    use super::{sqrt_rem_42, Word};
    use crate::UBig;

    #[test]
    fn test_sqrt_42() {
        let a = UBig::from_str_radix("100788288067706660892852085821456193179743392153874910688885216801600345870807", 10).unwrap();
        let mut a: [Word; 4] = a.as_words().try_into().unwrap();
        let mut b: [Word; 2] = [0, 0];
        let c = sqrt_rem_42(&mut b, &mut a);

        let r = UBig::from_words(&a[..2]);
        let s = UBig::from_words(&b);
        assert!(c);
        assert_eq!(s, UBig::from(317471712232297416216550966658362741242u128));
        assert_eq!(r, UBig::from(207656855896179259254063594487929956787u128));

        let a = (UBig::ONE << 256) - UBig::ONE;
        let mut a: [Word; 4] = a.as_words().try_into().unwrap();
        let mut b: [Word; 2] = [0, 0];
        let c = sqrt_rem_42(&mut b, &mut a);

        let r = UBig::from_words(&a[..2]);
        let s = UBig::from_words(&b);
        assert!(c);
        assert_eq!(s, UBig::from(340282366920938463463374607431768211455u128));
        assert_eq!(r, UBig::from(340282366920938463463374607431768211454u128));
    }
}
