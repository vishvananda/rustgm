//! Retained ABI syntax selects a calling convention independently of safety.
use super::{Declarations, Error, Kind, List, NodeKey, Report};

/// A language gate permits declaration formation, never cursor storage or execution.
pub(super) fn variadic_definition(
    report: &Report,
    key: NodeKey,
    flags: crate::syntax::item::FunctionFlags,
) -> Result<(), Error> {
    let error = |message| Declarations::error(report, key, message);
    let requested = report.package.requested_c_variadic().map_err(error)?;
    permission(report, key, "c_variadic", requested)?;
    if flags.safety != crate::syntax::item::Safety::Unsafe {
        return Err(error(
            "functions with a C variable argument list must be unsafe",
        ));
    }
    if flags.constant {
        let requested = report.package.requested_const_c_variadic().map_err(error)?;
        permission(report, key, "const_c_variadic", requested)?;
    }
    Ok(())
}
fn permission(report: &Report, key: NodeKey, name: &str, requested: bool) -> Result<(), Error> {
    if requested {
        return Ok(());
    }
    let span = report
        .package
        .unit(key.file)
        .configured
        .ast
        .node(key.node)
        .span;
    let feature = crate::resolve::bindings::FeatureName::new(name)
        .map_err(|message| Declarations::error(report, key, message))?;
    if report
        .macro_allowances()
        .allows_at(key.file, span, &feature)?
    {
        return Ok(());
    }
    Err(Declarations::error(
        report,
        key,
        "variadic definition requires its actual language feature request",
    ))
}

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
        variadic && unit.configured.active[last.get() as usize],
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
    let unit = report.package.unit(key.file);
    let node = if let Kind::Group(literal) = unit.configured.ast.node(node).kind {
        literal
    } else {
        node
    };
    let key = NodeKey { node, ..key };
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
        b"rust-call" if report.package.permits_source_rust_call() => Abi::RustCall,
        b"C" => Abi::C,
        b"unadjusted" => Abi::Unadjusted,
        _ => return Err(Declarations::error(report, key, "unsupported callable ABI")),
    };
    Ok(Convention {
        abi,
        variadic: false,
    })
}
