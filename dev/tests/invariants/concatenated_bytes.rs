//! Actual argument parses feed bounded byte output; no generated spelling is reparsed.
use super::*;
use crate::{
    expand::{crate_syntax, macros::input::Input},
    syntax::{
        expression::fragment::parse_arguments, interpolation::EMPTY_INPUTS, token_tree::Forest,
    },
};
use std::cell::Cell;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
fn produce(
    source: &str,
    charge: impl FnMut(usize, usize) -> std::result::Result<(), LexError>,
) -> Result<builtin::Expansion> {
    let forest = Forest::parse(source).map_err(|error| error.message)?;
    let input = Input::from_forest(&forest, forest.roots()).map_err(|error| error.message)?;
    let parsed =
        parse_arguments(source, &input.tokens, &EMPTY_INPUTS).map_err(|error| error.message)?;
    let config = Config::default();
    let configured =
        crate_syntax::configure(source, parsed, &config).map_err(|error| error.message)?;
    Ok(expand(
        &configured.ast,
        configured.ast.root,
        |index| {
            configured
                .ast
                .literals
                .get(index as usize)
                .cloned()
                .ok_or_else(|| error(0, "literal"))
        },
        charge,
        MacroContext::Expression,
        &config,
    )
    .map_err(|error| error.message)?)
}
fn bytes(value: &builtin::Expansion) -> Result<&[u8]> {
    let Kind::Literal(index) = value.syntax.ast.node(value.syntax.ast.root).kind else {
        return Err("byte result is not an actual literal".into());
    };
    let Value::ByteString(bytes) = &value.syntax.ast.literals[index as usize] else {
        return Err("byte result changed its typed literal kind".into());
    };
    Ok(bytes)
}
#[test]
fn eager_byte_shapes_preserve_zero_non_utf8_and_actual_repeat_count_magnitudes() -> Result {
    for (source, expected) in [
        ("b'A',b\"B\\0\\xff\",[67,68u8]", b"AB\0\xffCD".as_slice()),
        ("[b'X';2u8], [65;0]", b"XX".as_slice()),
        ("", b"".as_slice()),
    ] {
        let value = produce(source, |_, _| Ok(()))?;
        assert_eq!(bytes(&value)?, expected);
    }
    for source in [
        "65",
        "[65i8]",
        "[256]",
        "[256;0]",
        "[[65]]",
        "[b\"A\"]",
        "[b'A';1+1]",
        "(b\"A\")",
        "c\"A\"",
    ] {
        assert!(produce(source, |_, _| Ok(())).is_err(), "{source}");
    }
    Ok(())
}
#[test]
fn byte_repeat_and_overflow_attempts_are_charged_before_any_output_copy() {
    for source in [
        "[b'A';100000]",
        "[b'A';340282366920938463463374607431768211455u128]",
    ] {
        let attempted = Cell::new(0_usize);
        let failed = Cell::new(false);
        let mut charge = |_: usize, storage: usize| {
            attempted.set(attempted.get().saturating_add(storage));
            failed.set(failed.get() || storage > 128);
            if failed.get() {
                Err(error(0, "test byte budget failed"))
            } else {
                Ok(())
            }
        };
        assert!(produce(source, &mut charge).is_err());
        assert!(attempted.get() > 128);
        // The same mutable budget owner rejects an otherwise tiny retry too.
        assert!(produce("b'A'", &mut charge).is_err());
        assert!(failed.get());
    }
}
#[test]
fn byte_output_width_has_two_bounded_typed_passes_and_linear_payload() -> Result {
    for width in [8_usize, 32, 128] {
        let elements = std::iter::repeat_n("65", width)
            .collect::<Vec<_>>()
            .join(",");
        let source = format!("[{elements}]");
        let mut work = 0_usize;
        let mut payload = 0_usize;
        let result = produce(&source, |steps, bytes| {
            work += steps;
            payload += bytes;
            Ok(())
        })?;
        assert_eq!(bytes(&result)?.len(), width);
        assert!(work <= 4 * width + 32, "{width}: {work}");
        assert!(payload <= 64 * width + 2048, "{width}: {payload}");
        eprintln!("byte typed width={width} work={work} attempted-payload={payload}");
    }
    Ok(())
}
