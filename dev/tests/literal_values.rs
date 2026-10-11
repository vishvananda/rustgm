use rustgm::{
    lex::{
        Lexer,
        token::{Kind, Token},
    },
    literal::{
        interpret, render,
        value::{FloatBits, FloatType, Integer, IntegerType, Value},
    },
};
use std::borrow::Cow;

fn literal(source: &str) -> Result<Value<'_>, String> {
    let mut lexer = Lexer::new(source).map_err(|error| format!("{error:?}"))?;
    let token = lexer
        .next_token()
        .map_err(|error| format!("{error:?}"))?
        .ok_or("missing token")?;
    interpret(source, token)
        .map_err(|error| format!("{error:?}"))?
        .ok_or("not a literal".into())
}

#[test]
fn hand_checked_dump_is_a_value_view() -> Result<(), String> {
    let mut output = Vec::new();
    render::literals("42 0.1f32 \"é\" c\"x\"", &mut output).map_err(|e| format!("{e:?}"))?;
    assert_eq!(output, b"integer 0 2 infer 42\nfloat 3 9 f32 0x3dcccccd\nstring 10 14 2 \"\\xc3\\xa9\"\nc-string 15 19 2 \"x\\x00\"\neof 19\n");
    Ok(())
}

#[test]
fn signed_minimum_context_and_overflow_are_distinct() -> Result<(), String> {
    for (ty, max, minimum_bits) in [
        (IntegerType::I8, 127, 128),
        (IntegerType::I16, 32767, 32768),
        (IntegerType::I32, 2_147_483_647, 2_147_483_648),
        (
            IntegerType::I64,
            9_223_372_036_854_775_807,
            9_223_372_036_854_775_808,
        ),
        (
            IntegerType::Isize,
            9_223_372_036_854_775_807,
            9_223_372_036_854_775_808,
        ),
        (
            IntegerType::I128,
            170_141_183_460_469_231_731_687_303_715_884_105_727,
            170_141_183_460_469_231_731_687_303_715_884_105_728,
        ),
    ] {
        let value = |magnitude| Integer {
            magnitude,
            suffix: Some(ty),
        };
        assert!(!value(max).materialize(ty, false)?.overflow);
        assert!(value(max + 1).materialize(ty, false)?.overflow);
        let minimum = value(max + 1).materialize(ty, true)?;
        assert!(!minimum.overflow);
        assert_eq!(minimum.bits, minimum_bits);
        assert!(value(max + 2).materialize(ty, true)?.overflow);
        assert_eq!(value(0).materialize(ty, true)?.bits, 0);
    }
    Ok(())
}

#[test]
fn unsigned_widths_and_host_independence() -> Result<(), String> {
    for (ty, max) in [
        (IntegerType::U8, 255),
        (IntegerType::U16, 65535),
        (IntegerType::U32, 4_294_967_295),
        (IntegerType::U64, 18_446_744_073_709_551_615),
        (IntegerType::Usize, 18_446_744_073_709_551_615),
        (IntegerType::U128, u128::MAX),
    ] {
        let value = Integer {
            magnitude: max,
            suffix: Some(ty),
        };
        let result = value.materialize(ty, false)?;
        assert_eq!(result.bits, max);
        assert!(!result.overflow);
        assert!(value.materialize(ty, true).is_err());
        if let Some(magnitude) = max.checked_add(1) {
            let result = Integer {
                magnitude,
                suffix: None,
            }
            .materialize(ty, false)?;
            assert!(result.overflow);
            assert_eq!(result.bits, 0);
        }
    }
    Ok(())
}

#[test]
fn decoding_does_not_choose_an_inferred_type() -> Result<(), String> {
    let Value::Integer(integer) = literal("340282366920938463463374607431768211455")? else {
        return Err("expected integer".into());
    };
    assert_eq!(integer.magnitude, u128::MAX);
    assert_eq!(integer.suffix, None);
    assert_eq!(integer.materialize(IntegerType::U8, false)?.bits, 255);
    let Value::Float(float) = literal("1.0000000596046447753906250000000000000000001")? else {
        return Err("expected float".into());
    };
    assert_eq!(float.suffix, None);
    assert!(matches!(float.decimal, Cow::Borrowed(_)));
    assert_eq!(float.round(FloatType::F32)?, FloatBits::F32(0x3f80_0001));
    assert_eq!(
        float.round(FloatType::F64)?,
        FloatBits::F64(0x3ff0_0000_1000_0000)
    );
    Ok(())
}

#[test]
fn ordinary_and_raw_strings_borrow_the_source() -> Result<(), String> {
    for source in [
        "\"plain é\"",
        "r#\"plain é\"#",
        "b\"plain\"",
        "br#\"plain\"#",
        "c\"plain\"",
        "cr#\"plain\"#",
    ] {
        let value = literal(source)?;
        let (Value::String(bytes) | Value::ByteString(bytes) | Value::CString(bytes)) = value
        else {
            return Err("expected text".into());
        };
        assert!(matches!(bytes, Cow::Borrowed(_)));
        assert!(bytes.as_ptr() >= source.as_ptr());
        assert!(
            (bytes.as_ptr() as usize) + bytes.len() <= (source.as_ptr() as usize) + source.len()
        );
    }
    assert!(matches!(
        literal("\"escaped\\n\"")?,
        Value::String(Cow::Owned(_))
    ));
    Ok(())
}

#[test]
fn c_strings_preserve_opaque_bytes_and_implicit_nul() -> Result<(), String> {
    assert_eq!(
        literal(r#"c"\xe6""#)?,
        Value::CString(Cow::Borrowed(&[0xe6]))
    );
    assert_eq!(
        literal(r#"c"\u{e6}""#)?,
        Value::CString(Cow::Borrowed(&[0xc3, 0xa6]))
    );
    assert_eq!(literal("c\"\"")?, Value::CString(Cow::Borrowed(&[])));
    for source in [r#"c"\0""#, r#"c"\x00""#, r#"c"\u{0}""#, "cr\"\0\""] {
        assert!(literal(source).is_err());
    }
    Ok(())
}

#[test]
fn invalid_literal_retains_only_complete_prefix() {
    let mut output = Vec::new();
    assert!(render::literals("42 \"\\q\" 9", &mut output).is_err());
    assert_eq!(output, b"integer 0 2 infer 42\n");
}

#[test]
fn malformed_public_spans_are_errors() {
    for (start, end) in [(3, 0), (0, 999), (1, 2)] {
        assert!(
            interpret(
                "é",
                Token {
                    kind: Kind::Integer,
                    start,
                    end
                }
            )
            .is_err()
        );
    }
}

#[test]
fn large_inputs_use_iterative_decoding() -> Result<(), String> {
    let raw_source = format!("r#\"{}\"#", "a".repeat(1_000_000));
    let Value::String(bytes) = literal(&raw_source)? else {
        return Err("expected string".into());
    };
    assert_eq!(bytes.len(), 1_000_000);
    assert!(matches!(bytes, Cow::Borrowed(_)));
    let escaped = format!("\"{}\"", "\\u{1f980}".repeat(100_000));
    let Value::String(bytes) = literal(&escaped)? else {
        return Err("expected string".into());
    };
    assert_eq!(bytes.len(), 400_000);
    let malformed = format!("'\\u{{{}0}}'", "_".repeat(100_000));
    assert!(literal(&malformed).is_err());
    Ok(())
}

#[test]
fn deterministic_malformed_corpus_never_panics() -> Result<(), String> {
    let alphabet = [
        '0', '9', 'é', '\u{301}', '\0', '\r', '\n', '\'', '"', '\\', '#', '_', 'u', 'x', '{', '}',
    ];
    let mut seed = 99_u64;
    for _ in 0..2000 {
        let mut source = String::new();
        for _ in 0..64 {
            seed = seed.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
            source.push(alphabet[usize::try_from(seed >> 60).map_err(|e| e.to_string())?]);
        }
        let mut output = Vec::new();
        let rendered = render::literals(&source, &mut output);
        let counted = render::count(&source);
        assert_eq!(rendered.is_ok(), counted.is_ok());
        if let Ok(count) = counted {
            assert_eq!(
                std::str::from_utf8(&output)
                    .map_err(|e| e.to_string())?
                    .lines()
                    .count(),
                usize::try_from(count).map_err(|e| e.to_string())? + 1
            );
        }
    }
    Ok(())
}
