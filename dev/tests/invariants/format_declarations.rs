//! A recognized formatting header cannot execute or supply an allowance frame.
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
fn declared_format_roles_deny_before_real_input_capture_mark_and_invocation_work() -> TestResult {
    for name in [
        "literal-complete-basic",
        "literal-complete-const",
        "literal-complete-newline",
        "literal-complete-captured-expression",
    ] {
        let source = formatting_source(name)?;
        let probe = Probe::start()?;
        let error = formatting_expanded(&source)
            .err()
            .ok_or("formatting placeholder executed")?;
        assert!(
            error
                .to_string()
                .contains("formatting builtin invocation is not implemented"),
            "{name}: {error}"
        );
        let trace = probe.finish()?;
        let denied = trace
            .events
            .iter()
            .filter_map(|event| {
                if let Event::Denied(snapshot) = event {
                    Some(snapshot)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        assert_eq!(denied.len(), 1, "{name}");
        assert_eq!(
            denied[0].captures,
            usize::from(name == "literal-complete-captured-expression")
        );
        assert_eq!(
            denied[0].invocations,
            usize::from(name == "literal-complete-captured-expression")
        );
        assert!(denied[0].owner.is_some());
        if name != "literal-complete-captured-expression" {
            assert_eq!(denied[0].marks, 1);
        }
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
    assert!(roles.iter().all(|role| role.declaration_only()));
    for entry in &report.definitions {
        let owner = entry.node.ok_or("actual owner")?;
        let contract = crate::resolve::bindings::panic_contracts::published(
            &report.package,
            owner,
            entry.header.as_ref(),
            1,
        )?;
        assert!(contract.builtin.is_some_and(|role| role.declaration_only()));
    }
    Ok(())
}
