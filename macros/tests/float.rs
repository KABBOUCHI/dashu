use core::str::FromStr;

use dashu_float::{DBig, FBig};
use dashu_int::Word;
use dashu_macros::{dbig, fbig};

#[test]
fn test_fbig() {
    // binary digits
    assert_eq!(fbig!(0), FBig::ZERO);
    assert_eq!(fbig!(00001), FBig::ONE);
    assert_eq!(fbig!(-1.), FBig::NEG_ONE);
    assert_eq!(fbig!(-1.00), FBig::NEG_ONE);
    assert_eq!(fbig!(-101.001), FBig::from_str("-101.001").unwrap());
    assert_eq!(fbig!(1001.b23), FBig::from_str("1001.b23").unwrap());

    // hex digits
    assert_eq!(fbig!(0x1234), FBig::from_str("0x1234").unwrap());
    assert_eq!(fbig!(-_0x1.02), FBig::from_str("-0x1.02").unwrap());
    assert_eq!(fbig!(_0x1.), FBig::from_str("0x1.").unwrap());
    assert_eq!(fbig!(-_0x.02), FBig::from_str("-0x.02").unwrap());
    assert_eq!(fbig!(-_0x1.02p2), FBig::from_str("-0x1.02p2").unwrap());
    assert_eq!(fbig!(0x1p2), FBig::from_str("0x1p2").unwrap());
    assert_eq!(fbig!(_0x1.p - 2), FBig::from_str("0x1.p-2").unwrap());
    assert_eq!(fbig!(_0x.02p2), FBig::from_str("0x.02p2").unwrap());
    assert_eq!(fbig!(-_0x.02p-2), FBig::from_str("-0x.02p-2").unwrap());

    // const test
    const _: FBig = fbig!(0);
    const _: FBig = fbig!(1);
    const _: FBig = fbig!(-1);
    const _: FBig = fbig!(-10.01b100);
    #[cfg(target_pointer_width = "64")]
    {
        assert!(Word::BITS >= 64);
        const _: FBig = fbig!(0xffffffffffffffffp1234);
        const _: FBig = fbig!(-0xffffffffffffffffffffffffffffffffp-1234);
    }
}

#[test]
fn test_dbig() {
    assert_eq!(dbig!(0), DBig::ZERO);
    assert_eq!(dbig!(00001), DBig::ONE);
    assert_eq!(dbig!(-1.), DBig::NEG_ONE);
    assert_eq!(dbig!(-1.00), DBig::NEG_ONE);
    assert_eq!(dbig!(-123.004), DBig::from_str("-123.004").unwrap());

    assert_eq!(dbig!(1234.e23), DBig::from_str("1234.e23").unwrap());
    assert_eq!(dbig!(12.34e-5), DBig::from_str("12.34e-5").unwrap());

    // const test
    const _: DBig = dbig!(0);
    const _: DBig = dbig!(1);
    const _: DBig = dbig!(-1);
    const _: DBig = dbig!(-2.55e100);
    #[cfg(target_pointer_width = "64")]
    {
        assert!(Word::BITS >= 64);
        const _: DBig = dbig!(18446744073709551615e1234); // 2^64 * 10^1234
        const _: DBig = dbig!(-0.340282366920938463463374607431768211455e-1234); // 2^128 * 10^-(1234+128)
    }
}
