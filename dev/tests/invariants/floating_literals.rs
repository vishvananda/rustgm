//! Exact-bit hypotheses are independent of the candidate's rational conversion.
use super::{
    Value,
    value::{Float, FloatBits, FloatType},
};
use std::borrow::Cow;
type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn wide_decimals_match_frozen_ieee_boundaries_without_double_rounding() -> TestResult {
    let table = crate::test_source::fixture("pa2/tests/spec/floats/wide-bits.tsv")?;
    let mut count = 0;
    for row in table.lines().filter(|line| !line.starts_with('#')) {
        let columns: Vec<_> = row.split('\t').collect();
        assert_eq!(columns.len(), 4);
        let ty = if columns[1] == "f16" {
            FloatType::F16
        } else {
            FloatType::F128
        };
        let expected = u128::from_str_radix(columns[2], 16)?;
        let (negative, decimal) = columns[3]
            .strip_prefix('-')
            .map_or((false, columns[3]), |s| (true, s));
        let value = Float {
            decimal: Cow::Borrowed(decimal),
            suffix: Some(ty),
        };
        let sign = if negative {
            1_u128 << if ty == FloatType::F16 { 15 } else { 127 }
        } else {
            0
        };
        assert_eq!(value.round(ty)?.bits() ^ sign, expected, "{}", columns[0]);
        count += 1;
    }
    assert_eq!(count, 32);
    Ok(())
}

#[test]
fn numeric_shape_preserves_known_width_and_rejects_real_invalid_suffixes() -> TestResult {
    for (source, expected) in [
        ("1f16", FloatType::F16),
        (
            "340282366920938463463374607431768211456f128",
            FloatType::F128,
        ),
    ] {
        let token = crate::lex::Lexer::new(source)
            .map_err(|error| error.message)?
            .next_token()
            .map_err(|error| error.message)?
            .ok_or("literal token")?;
        let Some(Value::Float(value)) =
            super::interpret(source, token).map_err(|error| error.message)?
        else {
            return Err("known suffix was not float".into());
        };
        assert_eq!(value.suffix, Some(expected));
        assert!(matches!(value.decimal, Cow::Borrowed(_)));
    }
    for source in ["1.0f129", "0b1f128", "1.0f16suffix", "1e_f128"] {
        let token = crate::lex::Lexer::new(source)
            .map_err(|error| error.message)?
            .next_token()
            .map_err(|error| error.message)?
            .ok_or("negative token")?;
        assert!(super::interpret(source, token).is_err(), "{source}");
    }
    for decimal in ["", ".", "1..2", "1e", "1e+", "1f128"] {
        let value = Float {
            decimal: Cow::Borrowed(decimal),
            suffix: Some(FloatType::F128),
        };
        assert!(value.round(FloatType::F128).is_err(), "{decimal}");
    }
    assert_eq!(
        Float {
            decimal: Cow::Borrowed("0.1"),
            suffix: None
        }
        .round(FloatType::F32)?,
        FloatBits::F32(0x3dcc_cccd)
    );
    Ok(())
}
