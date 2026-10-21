//! Mutable indexing uses canonical inherited output and ordinary selected calls.
#[path = "../harness/foreign_frontend.rs"]
mod frontend;
use super::native;
use std::{
    path::{Path, PathBuf},
    process::Command,
};
fn root() -> Result<&'static Path, Box<dyn std::error::Error>> {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or_else(|| "root".into())
}
fn fixture(name: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
    Ok(root()?.join(format!("pa26/tests/spec/index-mut/{name}.rs")))
}
const ACCEPTED: &[&str] = &[
    "unsized",
    "closure-parameter",
    "store",
    "field",
    "borrow",
    "compound",
    "deref-chain",
    "nested",
    "generic",
    "alias",
    "shared",
    "lifetime",
    "raw-mut",
    "raw-const",
    "store-order",
    "compound-order",
    "reference-deref",
    "raw-deref-shared",
    "raw-field-shared",
];
const REJECTED: &[&str] = &[
    "escape",
    "captured",
    "immutable",
    "shared-base",
    "shared-deref",
    "missing-mut",
    "borrow-conflict",
    "index-conflict",
    "missing-supertrait",
    "extra-output",
    "wrong-reference",
    "lookalike",
    "reference-deref-missing",
    "compound-library-boundary",
];
#[test]
fn mutable_index_outcomes_match_independently_qualified_contracts()
-> Result<(), Box<dyn std::error::Error>> {
    let folder = root()?.join(format!(
        "obj/parallel/index-mut/source/{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&folder)?;
    let mut failures = Vec::new();
    for (&name, accepted) in ACCEPTED
        .iter()
        .map(|n| (n, true))
        .chain(REJECTED.iter().map(|n| (n, false)))
    {
        let input = fixture(name)?;
        let independent = Command::new("rustc")
            .args([
                "+1.98.0",
                "--edition=2024",
                "--crate-name=index_mut_case",
                "--cap-lints=allow",
                "--emit=obj",
            ])
            .arg(&input)
            .arg("-o")
            .arg(folder.join(format!("{name}.o")))
            .output()?;
        std::fs::write(folder.join(format!("{name}.stderr")), &independent.stderr)?;
        let rustc_accepted = accepted || matches!(name, "compound-library-boundary" | "captured");
        assert_eq!(
            independent.status.code(),
            Some(i32::from(!rustc_accepted)),
            "rustc {name}: {}",
            String::from_utf8_lossy(&independent.stderr)
        );
        let result = frontend::checked(&std::fs::read_to_string(input)?);
        if result.is_ok() != accepted {
            failures.push(format!("{name}: {:?}", result.err()));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}
#[test]
#[ignore = "requires the qualified external CPPGM LowIR backend"]
fn selected_mutable_index_places_and_evaluation_order_execute()
-> Result<(), Box<dyn std::error::Error>> {
    let names = [
        "store",
        "borrow",
        "compound",
        "nested",
        "generic",
        "lifetime",
        "raw-mut",
        "reference-deref",
        "raw-deref-shared",
        "raw-field-shared",
        "store-order",
        "compound-order",
        "unsized",
        "closure-parameter",
    ];
    let sources = names
        .iter()
        .map(|name| std::fs::read_to_string(fixture(name)?).map_err(Into::into))
        .collect::<Result<Vec<String>, Box<dyn std::error::Error>>>()?;
    let cases = names
        .iter()
        .zip(&sources)
        .map(|(&name, source)| {
            Ok((
                name,
                source
                    .strip_prefix(native::CHECK)
                    .ok_or("fixture check prefix")?,
                false,
            ))
        })
        .collect::<Result<Vec<_>, Box<dyn std::error::Error>>>()?;
    native::check_cases("parallel/pa26/index-mut/native", &cases)
}

#[test]
fn mutable_contract_inherits_the_single_canonical_index_output()
-> Result<(), Box<dyn std::error::Error>> {
    use rustgm::{resolve::model::BuiltinLanguageOutput, support::language_hooks::Hook};
    let source = std::fs::read_to_string(fixture("store")?)?;
    let (report, analysis) = frontend::analyze(&source)?;
    let index = report
        .resolution
        .builtin_traits
        .language(Hook::Index)
        .ok_or("Index")?;
    let mutable = report
        .resolution
        .builtin_traits
        .language(Hook::IndexMut)
        .ok_or("IndexMut")?;
    assert_ne!(index.definition, mutable.definition);
    assert!(matches!(index.output, BuiltinLanguageOutput::Declared(_)));
    assert_eq!(
        mutable.output,
        BuiltinLanguageOutput::Inherited {
            trait_definition: index.definition,
            declaration: index.output.definition()
        }
    );
    assert_eq!(mutable.output.declared(), None);
    let callable = analysis.facts.definitions[mutable.method.0 as usize].ok_or("method")?;
    let Some(rustgm::types::Kind::FunctionItem { signature, .. }) =
        analysis.facts.types.kind(callable)
    else {
        return Err("callable".into());
    };
    let Some(rustgm::types::Kind::Function {
        parameters,
        result,
        bound_regions,
        ..
    }) = analysis.facts.types.kind(signature)
    else {
        return Err("signature".into());
    };
    assert_eq!(bound_regions, 1);
    let receiver = *analysis
        .facts
        .types
        .list(parameters)
        .and_then(|p| p.first())
        .ok_or("receiver")?;
    for ty in [receiver, result] {
        assert!(matches!(
            analysis.facts.types.kind(ty),
            Some(rustgm::types::Kind::Reference { mutable: true, .. })
        ));
    }
    Ok(())
}
