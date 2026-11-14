//! Scaled finite ratios preserve IEEE boundary values across the whole exponent range.
use super::{Format, ratio};
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
