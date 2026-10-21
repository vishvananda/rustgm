//! Assignment traits retain source-level receiver order and ordinary calls.
use super::{mutation::frontend, native};
use std::path::{Path, PathBuf};
fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("../pa26/tests/spec/assignment/{name}.rs"))
}
const ACCEPTED: &[&str] = &[
    "add",
    "sub",
    "mul",
    "div",
    "rem",
    "and",
    "or",
    "xor",
    "shl",
    "shr",
    "default",
    "generic",
    "alias",
    "mixed",
    "reservation",
    "field",
    "index",
    "reference",
    "raw",
    "trait-order",
    "primitive-order",
    "generic-order",
    "unit",
    "mixed-unsuffixed",
];
const REJECTED: &[&str] = &[
    "immutable",
    "shared",
    "wrong-receiver",
    "wrong-result",
    "extra-output",
    "missing",
    "wrong-rhs",
    "lookalike",
    "move",
    "borrow-conflict",
    "generic-primitive-boundary",
];
#[test]
fn assignment_contracts_match_independent_outcomes() -> Result<(), Box<dyn std::error::Error>> {
    let mut failures = Vec::new();
    for (&name, accepted) in ACCEPTED
        .iter()
        .map(|n| (n, true))
        .chain(REJECTED.iter().map(|n| (n, false)))
    {
        let source = std::fs::read_to_string(fixture(name))?;
        super::rustc::qualify(
            &format!("assignment-{name}"),
            &source,
            accepted || name == "generic-primitive-boundary",
        )?;
        let result = frontend::checked(&source);
        if result.is_ok() != accepted {
            failures.push(format!("{name}: {:?}", result.err()));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}
#[test]
#[ignore = "requires the qualified external CPPGM LowIR backend"]
fn assignment_traits_and_source_order_execute() -> Result<(), Box<dyn std::error::Error>> {
    let sources = ACCEPTED
        .iter()
        .map(|n| std::fs::read_to_string(fixture(n)))
        .collect::<Result<Vec<_>, _>>()?;
    let cases = ACCEPTED
        .iter()
        .zip(&sources)
        .map(|(&n, s)| {
            Ok((
                n,
                s.strip_prefix(native::CHECK).ok_or("check prefix")?,
                false,
            ))
        })
        .collect::<Result<Vec<_>, Box<dyn std::error::Error>>>()?;
    native::check_cases("parallel/pa26/assignment/native", &cases)
}

#[test]
fn assignment_metadata_has_unit_output_and_one_mutable_receiver_binder()
-> Result<(), Box<dyn std::error::Error>> {
    use rustgm::{
        resolve::model::BuiltinLanguageOutput,
        support::language_hooks::Hook,
        types::{Kind, TypeId},
    };
    for (name, kind) in [
        ("add", Hook::AddAssign),
        ("sub", Hook::SubAssign),
        ("mul", Hook::MulAssign),
        ("div", Hook::DivAssign),
        ("rem", Hook::RemAssign),
        ("and", Hook::BitAndAssign),
        ("or", Hook::BitOrAssign),
        ("xor", Hook::BitXorAssign),
        ("shl", Hook::ShlAssign),
        ("shr", Hook::ShrAssign),
    ] {
        let (report, analysis) = frontend::analyze(&std::fs::read_to_string(fixture(name))?)?;
        let declaration = report
            .resolution
            .builtin_traits
            .language(kind)
            .ok_or("trait")?;
        assert_eq!(declaration.output, BuiltinLanguageOutput::Unit);
        assert_eq!(declaration.output.definition(), None);
        assert_eq!(declaration.output.owner(declaration.definition), None);
        assert_eq!(declaration.output.declared(), None);
        let callable = analysis.facts.definitions[declaration.method.0 as usize].ok_or("method")?;
        let Some(Kind::FunctionItem { signature, .. }) = analysis.facts.types.kind(callable) else {
            return Err("callable".into());
        };
        let Some(Kind::Function {
            parameters,
            result,
            bound_regions,
            ..
        }) = analysis.facts.types.kind(signature)
        else {
            return Err("signature".into());
        };
        assert_eq!(result, TypeId::UNIT);
        assert_eq!(bound_regions, 1);
        let parameters = analysis.facts.types.list(parameters).ok_or("parameters")?;
        assert_eq!(parameters.len(), 2);
        assert!(matches!(
            analysis.facts.types.kind(parameters[0]),
            Some(Kind::Reference { mutable: true, .. })
        ));
    }
    Ok(())
}
