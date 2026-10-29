//! Original associated equality remains a typed header obligation after derives.
use super::*;
fn input(name: &str) -> TestResult<String> {
    Ok(test_source::fixture(&format!(
        "pa31/tests/spec/associated-derive/{name}.rs"
    ))?)
}

#[test]
fn original_decode_equalities_survive_both_members_with_copied_identity() -> TestResult {
    let source = input("original-expansion")?;
    let report = expanded_provider(&source, "original_decode_equality")?;
    let entries = &report
        .attribute_effects
        .as_deref()
        .and_then(|effects| effects.derives.as_deref())
        .ok_or("derive list")?
        .entries;
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].contexts.len(), 2);
    report.expanded_derive(entries[0].owner, entries[0].effect)?;
    let unit = &report.package.units[0];
    let ast = &unit.configured.ast;
    let mut identities = Vec::new();
    for node in &ast.nodes {
        if let Kind::Constraint {
            target,
            value: Some(value),
            bounds,
        } = node.kind
        {
            let Kind::Name(_, name) = ast.node(target).kind else {
                return Err("actual associated target".into());
            };
            assert_eq!(bounds.start, bounds.end);
            let value = ast.node(value).span;
            assert_eq!(
                &unit.source.text()[value.start as usize..value.end as usize],
                "u16"
            );
            identities.push((node.span.start >= u32::try_from(source.len())?, name));
        }
    }
    let original = identities
        .iter()
        .find(|(generated, _)| !*generated)
        .ok_or("original equality")?
        .1;
    assert!(original.is_some());
    assert_eq!(
        identities
            .iter()
            .filter(|(generated, _)| *generated)
            .count(),
        2
    );
    assert!(identities.iter().all(|(_, name)| *name == original));
    assert!(!report.names_resolved);
    Ok(())
}

#[test]
fn source_clone_proves_associated_equality_and_actual_noncopy_field_body() -> TestResult {
    let source = input("generic-source")?;
    let report = test_source::source_provider(&source, "associated_clone_native")?;
    let crate::pipeline::Analysis {
        mut facts,
        prepared,
    } = crate::pipeline::analyze_source(&report)?;
    let program = crate::lower::instances::build_roots_prepared(
        &report,
        &mut facts,
        crate::lower::Roots {
            entry: None,
            exports: &[],
        },
        prepared,
    )?;
    let out = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../obj/parallel/associated-derive-v1/candidate");
    std::fs::create_dir_all(&out)?;
    std::fs::write(out.join("provider.rs"), &source)?;
    crate::lowir::writer::write(
        &program,
        &mut std::fs::File::create(out.join("provider.lowir"))?,
    )?;
    assert!(
        program
            .symbols
            .iter()
            .any(|symbol| symbol.name == "observed" && symbol.body.is_some())
    );
    for name in ["wrong-equality", "wrong-associated-name", "missing-clone"] {
        let source = input(name)?;
        let result =
            test_source::source_provider(&source, "associated_clone_opponent").and_then(|report| {
                crate::pipeline::analyze_source(&report)
                    .map(|_| ())
                    .map_err(|error| error.to_string())
            });
        let error = result.err().ok_or_else(|| format!("accepted {name}"))?;
        assert!(!error.contains("budget"), "{name}: {error}");
    }
    Ok(())
}
