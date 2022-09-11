use crate::repr::{Repr, Word};

#[inline]
pub(crate) fn check_inf<const B: Word>(repr: &Repr<B>) {
    if repr.is_infinite() {
        panic_operate_with_inf()
    }
}

#[inline]
pub(crate) fn check_inf_operands<const B: Word>(lhs: &Repr<B>, rhs: &Repr<B>) {
    if lhs.is_infinite() || rhs.is_infinite() {
        panic_operate_with_inf()
    }
}

/// Panics when operate with infinities
pub(crate) const fn panic_operate_with_inf() -> ! {
    panic!("arithmetic operations with the infinity are not allowed!")
}

pub(crate) fn check_precision_limited(precision: usize) {
    if precision == 0 {
        panic_unlimited_precision()
    }
}

/// Panics when operate on unlimited precision number
pub(crate) const fn panic_unlimited_precision() -> ! {
    panic!("precision cannot be 0 (unlimited) for this operation!")
}
