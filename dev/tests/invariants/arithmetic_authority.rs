//! Actual Add/Sub roles borrow existing source proof, body and scalar execution owners.
use super::*;
fn arithmetic_input(name: &str) -> TestResult<String> {
    Ok(crate::test_source::fixture(&format!(
        "pa31/tests/spec/source-add-sub/{name}.rs"
    ))?)
}
#[test]
fn actual_add_sub_headers_keep_exact_roles_without_bootstrap_factories() -> TestResult {
    let source = arithmetic_input("authored-bodies")?;
    let (report, Analysis { mut facts, .. }) = analyze(&source)?;
    assert!(facts.primitive_methods.is_none());
    assert_eq!(crate::support::bindings::LangItem::ALL.len(), 64);
    println!(
        "source-arithmetic-layout Reservations={} Catalog={} Body={} Operation={} Traits={} Resolution={}",
        size_of::<crate::resolve::bindings::Reservations>(),
        size_of::<crate::sema::compiler_items::Catalog>(),
        size_of::<crate::body::ir::Body>(),
        size_of::<Operation>(),
        size_of::<crate::resolve::model::BuiltinTraits>(),
        size_of::<crate::resolve::model::Resolution>()
    );
    for (name, hook, role, index) in [
        (
            "Add",
            Hook::Add,
            crate::support::bindings::LangItem::Add,
            62,
        ),
        (
            "Sub",
            Hook::Sub,
            crate::support::bindings::LangItem::Sub,
            63,
        ),
    ] {
        assert_eq!(role.index(), index);
        assert!(report.resolution.builtin_traits.language(hook).is_none());
        let definition = named(&report, name, DefKind::Trait)?;
        let site = report.resolution.definitions[definition.0 as usize]
            .node
            .ok_or("actual trait site")?;
        let before = facts.body_work.canonical_steps;
        let actual = source_binary(&report, &mut facts, site, hook)?;
        assert_eq!(actual.definition, definition);
        assert_eq!(
            facts.associated.trait_owner(actual.method),
            Some(definition)
        );
        assert!(matches!(
            facts.types.kind(actual.rhs),
            Some(Kind::Parameter(_))
        ));
        assert!(facts.body_work.canonical_steps > before);
    }
    let mut scalars = 0;
    let mut shared = 0;
    for (index, node) in report.package.units[0]
        .configured
        .ast
        .nodes
        .iter()
        .enumerate()
    {
        let Syntax::Binary {
            op: crate::lex::token::Punctuation::Plus | crate::lex::token::Punctuation::Minus,
            right,
            ..
        } = node.kind
        else {
            continue;
        };
        let key = NodeKey {
            file: crate::resolve::unit::FileId(0),
            node: NodeId::new(u32::try_from(index)?),
        };
        let Some(right) = facts
            .node(NodeKey { node: right, ..key })
            .and_then(|f| f.ty)
        else {
            continue;
        };
        if matches!(facts.types.kind(right), Some(Kind::Primitive(_))) {
            scalars += 1;
            assert!(!facts.methods.calls.contains_key(&key));
            assert!(matches!(
                facts.node(key).and_then(|f| f.operation),
                Some(Operation::Binary { .. })
            ));
        } else {
            shared += 1;
            assert!(facts.methods.calls.contains_key(&key));
            assert!(matches!(
                facts.node(key).and_then(|f| f.operation),
                Some(Operation::Call { .. })
            ));
        }
    }
    assert_eq!((scalars, shared), (2, 2));
    Ok(())
}
#[test]
fn actual_source_bodies_and_nominal_overloads_reach_original_native_inputs() -> TestResult {
    for (name, exports) in [
        (
            "explicit-shared-bodies",
            &["explicit_add", "explicit_sub", "shared_add", "shared_sub"][..],
        ),
        ("nominal-generic", &["overloaded", "subtract"][..]),
        (
            "macro-const-observed",
            &["observed_sum", "observed_difference", "observed_shared"][..],
        ),
    ] {
        let source = arithmetic_input(name)?;
        let (report, analysis) = analyze(&source)?;
        assert!(analysis.facts.primitive_methods.is_none());
        let program = lower(&report, analysis, exports)?;
        let folder = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../obj/parallel/pa31/source-add-sub-v1/candidate-runtime")
            .join(name);
        std::fs::create_dir_all(&folder)?;
        std::fs::write(folder.join("source.rs"), source)?;
        crate::lowir::writer::write(
            &program,
            &mut std::fs::File::create(folder.join("program.lowir"))?,
        )?;
        for &name in exports {
            assert_eq!(
                program
                    .symbols
                    .iter()
                    .filter(|s| s.name == name && s.body.is_some())
                    .count(),
                1
            );
        }
    }
    Ok(())
}
#[test]
fn missing_impl_type_and_const_opponents_cannot_reach_scalar_erasure() -> TestResult {
    for name in [
        "missing-add-impl",
        "missing-sub-impl",
        "same-name-no-role",
        "wrong-output",
        "wrong-rhs",
        "nonconst-primitive",
        "nonconst-explicit",
        "nonconst-overload",
        "role-wrong-target",
        "conditional-nonconst",
        "conditional-const",
    ] {
        let source = arithmetic_input(name)?;
        let result = crate::test_source::source_provider(&source, "source_arithmetic_negative")
            .and_then(|report| {
                pipeline::analyze_source(&report)
                    .map(|_| ())
                    .map_err(|e| e.to_string())
            });
        let error = result
            .err()
            .ok_or_else(|| format!("{name}: unproved actual source arithmetic accepted"))?;
        assert!(
            !error.contains("unsupported source language item"),
            "{name}: {error}"
        );
        println!("source-arithmetic-negative={name}: {error}");
    }
    Ok(())
}
#[test]
fn actual_const_impls_and_full_forwarding_macros_preserve_ctfe_authority() -> TestResult {
    for name in [
        "primitive-const",
        "conditional-unstable",
        "macro-const-scalar",
        "macro-const-with-location",
        "macro-const-observed",
    ] {
        let source = arithmetic_input(name)?;
        let (_, analysis) = analyze(&source)?;
        assert!(analysis.facts.primitive_methods.is_none());
    }
    for name in ["overflow-add", "overflow-sub"] {
        let source = arithmetic_input(name)?;
        let error = analyze(&source)
            .err()
            .ok_or("overflowing actual const operator accepted")?
            .to_string();
        assert!(error.contains("overflow"), "{name}: {error}");
        println!("source-arithmetic-const={name}: {error}");
    }
    Ok(())
}
#[test]
fn source_add_sub_header_opponents_replay_and_forged_signatures_stay_closed() -> TestResult {
    for name in ["header-reference-self", "header-wrong-default"] {
        let (report, Analysis { mut facts, .. }) = analyze(&arithmetic_input(name)?)?;
        let definition = named(&report, "Add", DefKind::Trait)?;
        let site = report.resolution.definitions[definition.0 as usize]
            .node
            .ok_or("source")?;
        assert!(source_binary(&report, &mut facts, site, Hook::Add).is_err());
    }
    let original = arithmetic_input("explicit-shared-bodies")?;
    let (mut report, Analysis { mut facts, .. }) = analyze(&original)?;
    let definition = named(&report, "Add", DefKind::Trait)?;
    let site = report.resolution.definitions[definition.0 as usize]
        .node
        .ok_or("source")?;
    let actual = source_binary(&report, &mut facts, site, Hook::Add)?;
    let old = facts.definitions[actual.method.0 as usize].replace(TypeId::UNIT);
    assert!(source_binary(&report, &mut facts, site, Hook::Add).is_err());
    facts.definitions[actual.method.0 as usize] = old;
    report.package.units[0].source =
        crate::support::source::SourceFile::new(original.into_bytes())?;
    assert!(source_binary(&report, &mut facts, site, Hook::Add).is_err());
    Ok(())
}

#[test]
fn source_member_code_uses_instance_proof_and_plain_const_code_keeps_its_owner() -> TestResult {
    let source = arithmetic_input("conditional-unstable")?;
    let report = crate::test_source::source_provider(&source, "source_member_instances")?;
    let definition = named(&report, "explicit", DefKind::Function)?;
    let (analysis, bodies) =
        crate::pipeline::execution::materialized_bodies::during(&[definition], || {
            pipeline::analyze_source(&report)
        })?;
    assert!(!bodies.is_empty());
    assert!(bodies.iter().all(|b| {
        b.instance.is_some()
            && !b
                .values
                .iter()
                .any(|v| crate::body::dispatch::trait_function(&analysis.facts, v))
    }));
    println!(
        "source const member Code instances={} original-template-owner={}",
        bodies.len(),
        crate::body::templates::owns(&analysis.facts, definition)
    );
    assert!(!crate::body::templates::owns(&analysis.facts, definition));
    let report = crate::test_source::resolve(
        "const fn plain()->u8{2+3}const VALUE:u8=plain();fn main(){let _=VALUE;}",
        true,
    )?;
    let definition = named(&report, "plain", DefKind::Function)?;
    let (analysis, bodies) =
        crate::pipeline::execution::materialized_bodies::during(&[definition], || {
            pipeline::analyze_source(&report)
        })?;
    assert!(analysis.facts.source_bindings.is_none());
    assert!(analysis.facts.primitive_methods.is_none());
    assert!(!bodies.is_empty());
    assert!(bodies.iter().all(|b| b.instance.is_none()));
    println!(
        "plain const Code originals={} source-catalog=false",
        bodies.len()
    );
    Ok(())
}
