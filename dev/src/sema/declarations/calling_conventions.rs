//! Retained ABI syntax selects a calling convention independently of safety.
use super::{Declarations, Error, Kind, List, NodeKey, Report};

/// The parser permits an ellipsis only at the end of the parameter list.
/// Keep that marker out of typed parameters without scanning ordinary inputs.
pub(in crate::sema) fn fixed_parameters(
    report: &Report,
    key: NodeKey,
    mut parameters: List,
) -> (List, bool) {
    let unit = report.package.unit(key.file);
    let Some(&last) = unit.configured.ast.list(parameters).last() else {
        return (parameters, false);
    };
    let mut node = last;
    while let Kind::Attributed { value, .. } = unit.configured.ast.node(node).kind {
        node = value;
    }
    let variadic = match unit.configured.ast.node(node).kind {
        Kind::Variadic => true,
        Kind::Parameter { ty: Some(ty), .. } => {
            matches!(unit.configured.ast.node(ty).kind, Kind::Variadic)
        }
        _ => false,
    };
    if variadic {
        parameters.end -= 1;
    }
    (
        parameters,
        variadic && unit.configured.active[last.0 as usize],
    )
}

pub(in crate::sema) fn with_variadic(
    report: &Report,
    key: NodeKey,
    mut convention: crate::types::callable::Convention,
    variadic: bool,
) -> Result<crate::types::callable::Convention, Error> {
    if variadic && convention.abi != crate::types::callable::Abi::C {
        return Err(Declarations::error(
            report,
            key,
            "variadic signatures require the C calling convention",
        ));
    }
    convention.variadic = variadic;
    Ok(convention)
}

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
