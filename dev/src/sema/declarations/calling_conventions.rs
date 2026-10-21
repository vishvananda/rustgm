//! Retained ABI syntax selects a calling convention independently of safety.
use super::{Declarations, Error, Kind, NodeKey, Report};

/// Trait methods and destructors require the ordinary Rust convention.
pub(super) fn rust_abi(
    report: &Report,
    key: NodeKey,
    abi: crate::syntax::item::Abi,
) -> Result<bool, Error> {
    Ok(callable_convention(report, key, abi)?.abi == crate::types::callable::Abi::Rust)
}
pub(in crate::sema) fn callable_convention(
    report: &Report,
    key: NodeKey,
    abi: crate::syntax::item::Abi,
) -> Result<crate::types::callable::Convention, Error> {
    use crate::types::callable::{Abi, Convention};
    let Some(node) = abi.literal() else {
        return Ok(Convention {
            abi: if abi == crate::syntax::item::Abi::Rust {
                Abi::Rust
            } else {
                Abi::C
            },
            variadic: false,
        });
    };
    let key = NodeKey { node, ..key };
    let unit = report.package.unit(key.file);
    let Kind::Literal(index) = unit.configured.ast.node(node).kind else {
        return Err(Declarations::error(
            report,
            key,
            "ABI requires a string literal",
        ));
    };
    let value = unit
        .literal(index)
        .map_err(|e| Declarations::error(report, key, e.message))?;
    let crate::literal::Value::String(bytes) = value else {
        return Err(Declarations::error(
            report,
            key,
            "ABI requires a string literal",
        ));
    };
    let abi = match bytes.as_ref() {
        b"Rust" => Abi::Rust,
        b"C" => Abi::C,
        _ => return Err(Declarations::error(report, key, "unsupported callable ABI")),
    };
    Ok(Convention {
        abi,
        variadic: false,
    })
}
