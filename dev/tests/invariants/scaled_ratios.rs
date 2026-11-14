//! Scaled finite ratios preserve IEEE boundary values across the whole exponent range.
use super::{Format, Unsigned, product, ratio};
#[test]
fn products_keep_every_carry_and_round_both_even_ties() -> Result<(), &'static str> {
    let full = Unsigned::product(u128::MAX, u128::MAX)?;
    let mut expected = Unsigned::from_u128(u128::MAX - 1);
    expected.shift_left(128)?;
    expected.multiply_add(1, 1)?;
    assert_eq!(full.compare(&expected), std::cmp::Ordering::Equal);
    assert_eq!(full.bit_len(), 256);
    assert!(full.reserved_words() <= 8);
    for (format, middle) in [
        (
            Format {
                precision: 11,
                maximum: 15,
            },
            0x3e00,
        ),
        (
            Format {
                precision: 113,
                maximum: 16_383,
            },
            0x3fff_8000_0000_0000_0000_0000_0000_0000,
        ),
    ] {
        let fraction = format.precision - 1;
        let hidden = 1_u128 << fraction;
        let scale = -i64::from(fraction) - 1;
        assert_eq!(product(hidden + 1, 3, scale, format)?, middle + 2);
        assert_eq!(product(hidden + 3, 3, scale, format)?, middle + 4);
        let minimum = 1 - format.maximum - i64::from(fraction);
        assert_eq!(product(1, 1, minimum - 1, format)?, 0);
        assert_eq!(product(3, 1, minimum - 1, format)?, 2);
        assert_eq!(product(hidden, hidden, 40_000, format)?, format.infinity()?);
        assert_eq!(product(hidden, hidden, -40_000, format)?, 0);
        assert_eq!(product(0, u128::MAX, 40_000, format)?, 0);
    }
    Ok(())
}
#[test]
fn ratios_round_subnormal_ties_and_saturate_extreme_exponents() -> Result<(), &'static str> {
    for (format, third) in [
        (
            Format {
                precision: 11,
                maximum: 15,
            },
            0x3555,
        ),
        (
            Format {
                precision: 113,
                maximum: 16_383,
            },
            0x3ffd_5555_5555_5555_5555_5555_5555_5555,
        ),
    ] {
        let minimum = 1 - format.maximum - i64::from(format.precision - 1);
        assert_eq!(ratio(1, 3, 0, format)?, third);
        assert_eq!(ratio(1, 2, minimum, format)?, 0);
        assert_eq!(ratio(3, 2, minimum, format)?, 2);
        assert_eq!(ratio(1, 1, format.maximum + 1, format)?, format.infinity()?);
        assert_eq!(ratio(1, 1, minimum - 2, format)?, 0);
        let largest = (1_u128 << format.precision) - 1;
        assert_eq!(ratio(largest, 1, 32_765, format)?, format.infinity()?);
        assert_eq!(ratio(1, largest, -32_765, format)?, 0);
        assert!(ratio(1, 0, 0, format).is_err());
    }
    Ok(())
}
