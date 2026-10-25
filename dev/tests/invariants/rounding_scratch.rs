//! Long literal tails retain bounded exact-conversion scratch.
use super::{Format, decimal};

#[test]
fn significant_prefix_scratch_stops_growing_for_long_zero_and_sticky_tails()
-> Result<(), &'static str> {
    let format = Format {
        precision: 113,
        maximum: 16383,
    };
    let mut maximum = 0;
    for width in [32, 128, 2048, 32768] {
        let spelling = format!("1.{}1", "0".repeat(width));
        let (digits, _, _) = decimal::scan(&spelling, format)?;
        let capacity = digits.reserved_words();
        assert!(
            capacity <= 2048,
            "{width} digits allocated {capacity} words"
        );
        maximum = maximum.max(capacity);
        if width >= 128 {
            assert_eq!(super::round(&spelling, false)?, 0x3fff_u128 << 112);
        }
        println!(
            "wide prefix source_digits={} reserved_words={}",
            spelling.len(),
            capacity
        );
    }
    assert!(maximum > 0);
    assert_eq!(
        super::round("1e+999999999999999999999", false)?,
        0x7fff_u128 << 112
    );
    assert_eq!(super::round("0e+999999999999999999999", false)?, 0);
    Ok(())
}
