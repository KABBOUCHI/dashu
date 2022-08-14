# Changelog

## 0.2.0 (WIP)

### Add

- Add a public API `as_words` and `as_sign_words` to access internal representation of `UBig` and `IBig`.
- Add const constructors `from_word`, `from_dword` and a direct constructor `from_words` for `UBig` and `IBig`.
- Implement `Mul` between `Sign` and `UBig`/`IBig`.
- Implement integer logarithm `ilog`.
- Implement approximated bounds of base 2 logarithm `log2_bounds`.

### Remove
- Remove `ubig!` and `ibig!` macros from the crate, more powerful versions of them will be included in a separate `dashu-macro` crate.

### Change

- Function `zero()`, `one()`, `neg_one()` are changed to associated constants `ZERO`, `ONE`, `NEG_ONE`.
- Parsing a string with unsupported radix will now return an Err instead of `panic!`.
- `from_str_with_radix_prefix` now also return the radix.

### Improve
- Parsing integers from string support underscore separater.
- Improve speed for power function `pow()`

## 0.1.1

### Add

- Implemented modular inverse for the `Modulo` type.
- Implemented `GCD` and `ExtendedGCD` traits for `UBig` and `IBig`.

## 0.1.0

The code for big integer is ported from `ibig @ 0.3.5` with modifications stated in the [NOTICE](./NOTICE.md), the current MSRV for `dashu-int` is 1.57.

# Todo

- Division: trim the trailing zero words before division.
- GCD: trim the trailing zero words, and give the number of zeros as input to low-level algorithms.
- GCD: An idea of fast gcd check for rational number: don't do gcd reduction after every operation.
  For small numerators or denominators, we can directly do a gcd, otherwise, we first do gcd with a primorial that
  fits in a word, and only remove these small divisors.
  Further improvement: store a const divisor for the prime factors in the primorial, thus supports a fast factorial of
  the gcd result, and then divide with these const divisor.
- Logarithm: for very large est value, the est error is large and there could be many fixing steps,
  we should use a similar strategy as the non_power_two formatter, using power sequence,
  or call log again on the target / est_pow
- Power: implement a k-ary pow when exponent is too large (after lifting to at least a full word), this will store pre-computed 2^1~2^k powers. Maybe move this implementation to a separate module folder, and use the window selection function from modular pow.
