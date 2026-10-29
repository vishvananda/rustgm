//! Actual v4 providers and fresh source clients retain whole non-type clauses.
use crate::{
    lower::instances::SourcePublication,
    module_packets as test, source_packets,
    support::crates::{
        self,
        generics::{self as g, source as s},
    },
};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
fn fixture(name: &str) -> Result<String> {
    Ok(crate::test_source::fixture(&format!(
        "pa31/tests/spec/feature-predicate-metadata-v4/{name}.rs"
    ))?)
}
fn provider(name: &str) -> Result<crates::Interface> {
    test::exported_profile(
        &fixture(name)?,
        SourcePublication::ModuleFeaturePredicates,
        "feature_predicate_v4",
    )
}
fn client(artifact: crates::Interface, name: &str) -> Result<crate::lowir::program::Program> {
    let (
        report,
        crate::pipeline::Analysis {
            mut facts,
            prepared,
        },
        _,
    ) = source_packets::client_analysis(artifact, &fixture(name)?)?;
    let index = report
        .resolution
        .definitions
        .iter()
        .position(|d| {
            d.kind == crate::resolve::model::DefKind::Function
                && report.package.names.get(d.name) == "answer"
        })
        .ok_or("actual answer export")?;
    let exports = [crate::resolve::model::DefId(u32::try_from(index)?)];
    Ok(crate::lower::instances::build_roots_prepared(
        &report,
        &mut facts,
        crate::lower::Roots {
            entry: None,
            exports: &exports,
        },
        prepared,
    )?)
}
fn save(
    artifact: &crates::Interface,
    program: &crate::lowir::program::Program,
    name: &str,
) -> Result {
    assert!(program.symbols.iter().any(|s| s.body.is_some()));
    let mut lowir = Vec::new();
    crate::lowir::writer::write(program, &mut lowir)?;
    assert!(!lowir.is_empty());
    let out = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../obj/parallel/feature-predicate-metadata-v4/candidate-v1")
        .join(name);
    std::fs::create_dir_all(&out)?;
    std::fs::write(out.join("consumer.lowir"), lowir)?;
    std::fs::write(out.join("provider.rgcrate"), crates::encode(artifact)?)?;
    Ok(())
}
#[test]
fn actual_provider_fresh_clients_keep_predicate_and_regular_api_independent() -> Result {
    let artifact = provider("provider")?;
    let bytes = crates::encode(&artifact)?;
    assert_eq!(bytes.get(13..15), Some([4, 0].as_slice()));
    assert_eq!(crates::encode(&crates::decode(&bytes)?)?, bytes);
    for name in ["ordinary-both", "staged-scope"] {
        let program = client(crates::decode(&bytes)?, name)?;
        save(&artifact, &program, name)?;
    }
    for (name, needle) in [
        ("ordinary-api-only", "feature"),
        ("ordinary-probe-only", "predicate_api"),
        ("unknown-bound-name", "undefined library feature"),
        ("staged-global-only", "feature"),
    ] {
        let error = client(crates::decode(&bytes)?, name)
            .err()
            .ok_or("opponent accepted")?
            .to_string();
        assert!(error.contains(needle), "{name}: {error}");
    }
    assert!(
        test::exported_profile(
            &fixture("provider")?,
            SourcePublication::ModuleAvailability,
            "old_v3"
        )
        .is_err()
    );
    Ok(())
}
#[test]
fn actual_primitive_identity_method_reconstructs_and_dispatches_in_imported_scope() -> Result {
    let artifact = provider("identity-method-provider")?;
    let bytes = crates::encode(&artifact)?;
    let program = client(crates::decode(&bytes)?, "identity-method-both")?;
    save(&artifact, &program, "identity-method-both")?;
    let omitted = changed(
        &artifact,
        |rows| {
            let types = &rows.types;
            for input in &mut rows.body_inputs {
                input.obligations.retain(|goal| {
                    !matches!(goal.kind, s::InputKind::WellFormed(root)
                if matches!(types.get(root.0 as usize), Some(s::Type::FunctionItem(_))))
                });
            }
        },
        0,
        &mut g::Work::default(),
    );
    assert!(
        omitted.is_err(),
        "a missing actual FunctionItem formation stays a hard data error"
    );
    assert!(client(crates::decode(&bytes)?, "identity-method-api-only").is_err());
    Ok(())
}
#[test]
fn actual_primitive_literal_method_reconstructs_and_dispatches_in_imported_scope() -> Result {
    let artifact = provider("method-provider")?;
    let bytes = crates::encode(&artifact)?;
    let program = client(crates::decode(&bytes)?, "method-both")?;
    save(&artifact, &program, "method-both")?;
    assert!(client(crates::decode(&bytes)?, "method-api-only").is_err());
    Ok(())
}
fn changed(
    artifact: &crates::Interface,
    mutate: impl FnOnce(&mut s::SourceRows),
    base: usize,
    work: &mut g::Work,
) -> Result<g::Payload> {
    let (mut rows, attrs, modules) = test::raw(artifact)?;
    mutate(&mut rows);
    let availability = artifact
        .source_descriptor()
        .ok_or("descriptor")?
        .availability()?
        .clone();
    Ok(g::Payload::from_source_module_feature_predicates(
        artifact.identity(),
        rows,
        attrs,
        modules,
        availability,
        base,
        work,
    )?)
}
#[test]
fn malformed_non_type_rows_and_legacy_profiles_cannot_drop_or_invent_debt() -> Result {
    let artifact = provider("provider")?;
    let changes: [fn(&mut s::SourceRows); 7] = [
        |rows| {
            if let Some(c) = rows
                .clauses
                .iter_mut()
                .find(|c| matches!(c.requirement, s::Requirement::UnstableFeature(_)))
            {
                c.requirement = s::Requirement::UnstableFeature("".into());
            }
        },
        |rows| {
            if let Some(c) = rows
                .clauses
                .iter_mut()
                .find(|c| matches!(c.requirement, s::Requirement::UnstableFeature(_)))
            {
                c.requirement = s::Requirement::UnstableFeature("not-feature".into());
            }
        },
        |rows| {
            if let Some(c) = rows
                .clauses
                .iter_mut()
                .find(|c| matches!(c.requirement, s::Requirement::UnstableFeature(_)))
            {
                c.requirement = s::Requirement::UnstableFeature("x".repeat(257).into());
            }
        },
        |rows| {
            if let Some(c) = rows
                .clauses
                .iter_mut()
                .find(|c| matches!(c.requirement, s::Requirement::UnstableFeature(_)))
            {
                c.owner = crates::owner_catalog::DeclarationRef(0);
            }
        },
        |rows| {
            if let Some(c) = rows
                .clauses
                .iter_mut()
                .find(|c| matches!(c.requirement, s::Requirement::UnstableFeature(_)))
            {
                c.binder = Some(g::BinderRef(0));
            }
        },
        |rows| {
            if let Some(c) = rows
                .clauses
                .iter_mut()
                .find(|c| matches!(c.requirement, s::Requirement::UnstableFeature(_)))
            {
                c.self_subject = Some(s::SlotRef(0));
            }
        },
        |rows| {
            if let Some(c) = rows
                .clauses
                .iter_mut()
                .find(|c| matches!(c.requirement, s::Requirement::UnstableFeature(_)))
            {
                c.site = u32::MAX;
            }
        },
    ];
    for change in changes {
        assert!(changed(&artifact, change, 0, &mut g::Work::default()).is_err());
    }
    let mut failed = g::Work::default();
    assert!(changed(&artifact, |_| {}, crates::MAX_BYTES, &mut failed).is_err());
    assert!(failed.steps > 0);
    assert_eq!(size_of::<s::SourceRows>(), 640);
    assert_eq!(size_of::<s::Columns>(), 616);
    assert_eq!(size_of::<crate::types::parameters::clauses::Clause>(), 12);
    println!(
        "v4 cold requirement={} clause={} located={} dense={}",
        size_of::<s::Requirement>(),
        size_of::<s::Clause>(),
        size_of::<s::LocatedRequirement>(),
        size_of::<crate::sema::model::NodeFacts>()
    );
    Ok(())
}

#[test]
fn private_original_names_bind_actual_provider_body_and_declaration() -> Result {
    let mut results = Vec::new();
    for disambiguator in ["private_scope_a", "private_scope_b"] {
        let artifact = test::exported_profile(
            &fixture("identity-method-provider")?,
            SourcePublication::ModuleFeaturePredicates,
            disambiguator,
        )?;
        let rows = artifact
            .source_descriptor()
            .ok_or("actual descriptor")?
            .rows();
        let function = rows
            .functions
            .iter()
            .find(|function| {
                function.parent.is_some_and(|parent| {
                    matches!(
                        rows.header(parent),
                        Ok(Some(s::HeaderRef::Implementation(_)))
                    )
                })
            })
            .ok_or("actual private impl method")?;
        let s::FunctionBody::Original(body) = function.body else {
            return Err("actual private body".into());
        };
        let coordinates = (body, function.declaration);
        let expected = artifact
            .identity()
            .private_original_symbol(body, function.declaration);
        let bytes = crates::encode(&artifact)?;
        let (report, analysis, _) = source_packets::client_analysis(
            crates::decode(&bytes)?,
            &fixture("identity-method-both")?,
        )?;
        let index = report
            .resolution
            .definitions
            .iter()
            .enumerate()
            .find_map(|(index, d)| {
                let definition = crate::resolve::model::DefId(u32::try_from(index).ok()?);
                (d.kind == crate::resolve::model::DefKind::Function
                    && analysis
                        .facts
                        .associated
                        .owner(definition)
                        .and_then(|id| analysis.facts.associated.get(id))
                        .is_some_and(|owner| {
                            matches!(
                                owner.kind,
                                crate::sema::associated::OwnerKind::ImportedImpl(_)
                            )
                        }))
                .then_some(definition)
            })
            .ok_or("actual fresh imported impl method")?;
        assert_eq!(
            analysis
                .facts
                .crates
                .as_deref()
                .and_then(|linkage| linkage.rust_symbol(index)),
            Some(expected.as_str())
        );
        results.push((coordinates, expected, analysis.facts, index));
    }
    let (right_coordinates, right_symbol, mut right, _) = results.pop().ok_or("right")?;
    let (left_coordinates, left_symbol, mut left, definition) = results.pop().ok_or("left")?;
    assert_eq!(left_coordinates, right_coordinates);
    assert_ne!(left_symbol, right_symbol);
    left.crates = right.crates.take();
    assert!(
        left.function_header(definition).is_err(),
        "foreign catalog cannot acquire the same private coordinates"
    );
    Ok(())
}
