//! Union Copy is an ordinary marker impl; authored Clone copies the whole union.
use super::*;
use crate::syntax::{ast::Kind, item::ItemKind};
const FIXTURES: &str = "pa31/tests/spec/union-copy/";
fn fixture(name: &str) -> TestResult<String> {
    Ok(test_source::fixture(&format!("{FIXTURES}{name}.rs"))?)
}
#[test]
fn original_maybe_uninit_copy_preserves_real_union_fields_and_authored_clone() -> TestResult {
    let input = fixture("maybe-uninit-expansion")?;
    let report = expanded_provider(&input, "original_union")?;
    let entries = &report
        .attribute_effects
        .as_deref()
        .and_then(|e| e.derives.as_deref())
        .ok_or("actual derive")?
        .entries;
    assert_eq!(entries.len(), 1);
    let entry = &entries[0];
    report.expanded_derive(entry.owner, entry.effect)?;
    assert_eq!(entry.contexts.len(), 1);
    assert!(matches!(
        report.definitions[report.invocations[entry.contexts.start].definition].body,
        Body::Builtin(Builtin::Copy)
    ));
    let unit = &report.package.units[0];
    let ast = &unit.configured.ast;
    let mut owner = entry.owner.node;
    while let Kind::Attributed { value, .. } = ast.node(owner).kind {
        owner = value;
    }
    let Kind::Item(index) = ast.node(owner).kind else {
        return Err("actual union item".into());
    };
    let ItemKind::Union { fields } = ast.items[index as usize].kind else {
        return Err("union owner changed".into());
    };
    assert_eq!(ast.list(fields).len(), 2);
    let mut generated = 0;
    for node in &ast.nodes {
        if (node.span.start as usize) < input.len() {
            continue;
        }
        let Kind::Item(index) = node.kind else {
            continue;
        };
        let item = ast.items[index as usize];
        if let ItemKind::Impl {
            items,
            trait_path: Some(_),
            ..
        } = item.kind
        {
            assert!(ast.list(items).is_empty());
            assert_eq!(ast.list(item.generics).len(), 1);
            generated += 1;
        }
    }
    assert_eq!(generated, 1);
    assert!(input.contains("// Not calling `T::clone()`, we cannot know if we are initialized enough for that.\n        *self"));
    assert!(!report.names_resolved);
    Ok(())
}
#[test]
fn real_source_union_copies_initialized_bits_and_never_reads_uninitialized_field() -> TestResult {
    let input = fixture("source-runtime")?;
    let report = test_source::source_provider(&input, "union_copy")?;
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
    assert!(
        program
            .symbols
            .iter()
            .any(|s| s.name == "observed" && s.body.is_some())
    );
    let out = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../obj/parallel/union-copy-v1/candidate/native");
    std::fs::create_dir_all(&out)?;
    std::fs::write(out.join("provider.rs"), input)?;
    crate::lowir::writer::write(
        &program,
        &mut std::fs::File::create(out.join("provider.lowir"))?,
    )?;
    Ok(())
}
#[test]
fn actual_copy_clone_field_type_privacy_and_unsafe_obligations_remain_required() -> TestResult {
    for name in [
        "missing-clone",
        "wrong-copy-trait",
        "noncopy-field",
        "private-field",
        "safe-field",
    ] {
        let error = test_source::source_provider(&fixture(name)?, "union_opponent")
            .and_then(|report| {
                let crate::pipeline::Analysis {
                    mut facts,
                    prepared,
                } = crate::pipeline::analyze_source(&report).map_err(|e| e.to_string())?;
                // The privacy opponent has an ordinary Rust body rather than a
                // native export. Demand it explicitly; latent acceptance is not
                // evidence that the private field can be accessed.
                let exports = if name == "private-field" {
                    let definition = report
                        .resolution
                        .definitions
                        .iter()
                        .position(|definition| {
                            definition.kind == crate::resolve::model::DefKind::Function
                                && report.package.names.get(definition.name) == "read"
                        })
                        .ok_or("missing actual privacy opponent body")?;
                    vec![crate::resolve::model::DefId(
                        u32::try_from(definition).map_err(|e| e.to_string())?,
                    )]
                } else {
                    Vec::new()
                };
                crate::lower::instances::build_roots_prepared(
                    &report,
                    &mut facts,
                    crate::lower::Roots {
                        entry: None,
                        exports: &exports,
                    },
                    prepared,
                )
                .map(|_| ())
                .map_err(|e| e.to_string())
            })
            .err()
            .ok_or_else(|| format!("accepted {name}"))?;
        assert!(!error.contains("budget"), "{name}: {error}");
        assert!(!error.contains("wider profile"), "{name}: {error}");
        eprintln!("union Copy opponent {name}: {error}");
    }
    for name in ["clone-required", "debug-required-expansion"] {
        let error = expanded_provider(&fixture(name)?, "union_required")
            .err()
            .ok_or_else(|| format!("accepted {name}"))?
            .to_string();
        assert!(
            error
                .contains("source union derives other than Copy require their actual union recipe"),
            "{name}: {error}"
        );
    }
    Ok(())
}
#[test]
fn union_field_inventory_work_and_rejected_families_preserve_transactional_payment() -> TestResult {
    use std::fmt::Write as _;
    let input = fixture("source-runtime")?;
    let mut previous = None;
    for count in [8, 32, 128] {
        let mut source = input.clone();
        source.push_str("#[derive(crate::marker::Copy)]union Many<const N:usize=3>{");
        for field in 0..count {
            write!(&mut source, "f{field}:u32,")?;
        }
        source.push('}');
        let report = expanded_provider(&source, "many_union_fields")?;
        let payer = &report.package.source_core().ok_or("actual payer")?.macros;
        let before = payer.usage();
        let entries = &report
            .attribute_effects
            .as_deref()
            .and_then(|e| e.derives.as_deref())
            .ok_or("actual derives")?
            .entries;
        for entry in entries {
            report.expanded_derive(entry.owner, entry.effect)?;
        }
        let after = payer.usage();
        assert!(after.0 > before.0);
        assert_eq!(after.1, before.1);
        if let Some((work, bytes)) = previous {
            assert!(after.0 <= work * 6, "fields={count}: {after:?}");
            assert!(after.1 <= bytes * 6, "fields={count}: {after:?}");
        }
        previous = Some((after.0, after.1));
        eprintln!("source union fields={count}: {after:?}");
    }
    let source = fixture("debug-required-expansion")?;
    super::super::super::role_declarations::with_engine(&source, true, |engine| {
        let before = engine.loader.package.units[0].source.text().len();
        assert!(engine.run().is_err());
        assert_eq!(engine.loader.package.units[0].source.text().len(), before);
        Ok(())
    })?;
    assert_eq!(size_of::<crate::syntax::ast::Node>(), 36);
    assert_eq!(size_of::<crate::body::ir::Body>(), 456);
    Ok(())
}
