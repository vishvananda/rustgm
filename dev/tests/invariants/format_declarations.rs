//! Formatting headers and literal results retain their actual source and call identities.
use super::*;
fn formatting_source(name: &str) -> TestResult<String> {
    Ok(crate::test_source::fixture(&format!(
        "pa31/tests/spec/source-format-macros/{name}.rs"
    ))?)
}
fn formatting_expanded(source: &str) -> TestResult<crate::resolve::expansion::Report> {
    let identity = crate::support::crates::CrateIdentity::new("minicore", "format_declarations")?;
    Ok(crate::resolve::expansion::analyze_with_context(
        InputFile {
            identity: "format.rs".into(),
            path: "format.rs".into(),
            source: SourceFile::new(source.as_bytes().to_vec())?,
        },
        &mut NoFiles,
        &Config::default(),
        &crate::resolve::bindings::Context {
            crate_name: Some("minicore".into()),
            current: Some(identity.clone()),
            mode: crate::support::bindings::Mode::Source { provider: identity },
            ..Default::default()
        },
    )?)
}
#[test]
fn literal_format_roles_retain_real_owned_results_and_invocation_frames() -> TestResult {
    for name in [
        "literal-complete-basic",
        "literal-complete-const",
        "literal-complete-newline",
        "literal-complete-captured-expression",
    ] {
        let source = formatting_source(name)?;
        let report = formatting_expanded(&source)?;
        let values = report.package.units.iter().enumerate().flat_map(|(file, unit)| {
            unit.configured.ast.nodes.iter().enumerate().filter_map(move |(node, actual)| {
                (unit.configured.active[node] && matches!(actual.kind,
                    crate::syntax::ast::Kind::LibraryValue { kind: crate::support::library_types::LibraryValue::FormatLiteral { .. }, .. }
                )).then_some(crate::resolve::unit::NodeKey {
                    file: crate::resolve::unit::FileId(u32::try_from(file).ok()?),
                    node: crate::syntax::ast::NodeId::new(u32::try_from(node).ok()?),
                })
            })
        }).collect::<Vec<_>>();
        assert_eq!(values.len(), 1, "{name}");
        assert!(report.formatting_value(values[0])?.formatting());
    }
    Ok(())
}
#[test]
fn authentic_format_headers_preserve_real_roles_without_executable_frames() -> TestResult {
    let report = formatting_expanded(&formatting_source("declaration-original-three")?)?;
    assert_eq!(report.invocations.len(), 0);
    assert_eq!(report.captures.len(), 0);
    assert_eq!(report.package.marks.context_count(), 1);
    let roles = report
        .definitions
        .iter()
        .filter_map(|entry| {
            if let crate::resolve::expansion::Body::Builtin(role) = entry.body {
                Some(role)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    assert_eq!(roles.len(), 3);
    assert!(roles.iter().all(|role| role.formatting()));
    for entry in &report.definitions {
        let owner = entry.node.ok_or("actual owner")?;
        let contract = crate::resolve::bindings::panic_contracts::published(
            &report.package,
            owner,
            entry.header.as_ref(),
            1,
        )?;
        assert!(contract.builtin.is_some_and(|role| role.formatting()));
    }
    Ok(())
}
