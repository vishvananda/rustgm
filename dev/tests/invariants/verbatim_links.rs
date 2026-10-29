//! Version-five exact exports retain actual C signatures and external body ownership.
use crate::{
    lower::instances::{self, SourcePublication},
    module_packets as test, pipeline, source_packets,
    support::{
        crates::{
            self,
            generics::{self as g, source as s},
        },
        linkage::{Linkage, Naming},
    },
    types::{Kind, callable::Convention},
};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
fn fixture(name: &str) -> Result<String> {
    Ok(crate::test_source::fixture(&format!(
        "pa31/tests/spec/exact-source-linkage/{name}.rs"
    ))?)
}
fn provider(source: &str) -> Result<(crate::lowir::program::Program, crates::Interface)> {
    provider_named(source, "exact_source_linkage")
}
fn provider_named(
    source: &str,
    disambiguator: &str,
) -> Result<(crate::lowir::program::Program, crates::Interface)> {
    let report = crate::test_source::source_provider(source, disambiguator)?;
    let identity = report
        .package
        .source_core()
        .ok_or("provider")?
        .provider
        .clone();
    let pipeline::Analysis {
        mut facts,
        prepared,
    } = pipeline::analyze_source(&report)?;
    Ok(instances::build_source_library_profile(
        &report,
        &mut facts,
        identity,
        prepared,
        SourcePublication::ModuleDefinitions,
    )?)
}
fn write(name: &str, bytes: impl AsRef<[u8]>) -> Result {
    let out = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../obj/parallel/exact-source-linkage-v5/candidate-v1");
    std::fs::create_dir_all(&out)?;
    std::fs::write(out.join(name), bytes)?;
    Ok(())
}
#[test]
fn genuine_provider_and_fresh_client_keep_exact_c_definitions_separate() -> Result {
    let (program, artifact) = provider(&fixture("provider")?)?;
    let bytes = crates::encode(&artifact)?;
    assert_eq!(bytes.get(13..15), Some([5, 0].as_slice()));
    assert_eq!(crates::encode(&crates::decode(&bytes)?)?, bytes);
    assert_eq!(
        artifact
            .source_descriptor()
            .ok_or("descriptor")?
            .rows()
            .exact_callables
            .len(),
        7
    );
    let mut lowir = Vec::new();
    crate::lowir::writer::write(&program, &mut lowir)?;
    write("provider.lowir", lowir)?;
    write_distinct_provider(&artifact)?;
    let (
        report,
        pipeline::Analysis {
            mut facts,
            prepared,
        },
        _,
    ) = source_packets::client_analysis(crates::decode(&bytes)?, &fixture("consumer")?)?;
    let imported = report
        .resolution
        .definitions
        .iter()
        .enumerate()
        .filter(|(_, d)| {
            d.node.is_none()
                && [
                    "byte", "wide", "signed", "flag", "letter", "single", "double",
                ]
                .contains(&report.package.names.get(d.name))
        })
        .map(|(i, _)| u32::try_from(i).map(crate::resolve::model::DefId))
        .collect::<std::result::Result<Vec<_>, _>>()?;
    assert_eq!(imported.len(), 7);
    check_imported_originals(&facts, imported)?;
    let program = instances::build_roots_prepared(
        &report,
        &mut facts,
        crate::lower::Roots {
            entry: None,
            exports: &[],
        },
        prepared,
    )?;
    for name in [
        "byte", "wide", "signed", "flag", "letter", "single", "double",
    ] {
        assert_eq!(
            program
                .symbols
                .iter()
                .filter(|s| s.name == format!("source_{name}") && s.body.is_some())
                .count(),
            0
        );
        assert!(
            program
                .symbols
                .iter()
                .any(|s| s.name == name && s.body.is_some())
        );
    }
    let mut lowir = Vec::new();
    crate::lowir::writer::write(&program, &mut lowir)?;
    write("client.lowir", lowir)?;
    write("provider.rgcrate", bytes)?;
    Ok(())
}
fn write_distinct_provider(first: &crates::Interface) -> Result {
    let (program, second) = provider_named(&fixture("provider")?, "distinct_source_provider")?;
    assert_ne!(first.identity(), second.identity());
    assert_ne!(crates::encode(first)?, crates::encode(&second)?);
    let a = first.source_descriptor().ok_or("first provider")?.rows();
    let b = second.source_descriptor().ok_or("second provider")?.rows();
    assert_eq!(a.exact_callables, b.exact_callables);
    let mut lowir = Vec::new();
    crate::lowir::writer::write(&program, &mut lowir)?;
    write("other-provider.lowir", lowir)
}
fn check_imported_originals(
    facts: &crate::sema::model::Declarations,
    imported: Vec<crate::resolve::model::DefId>,
) -> Result {
    for definition in imported {
        let record = facts
            .foreign
            .as_deref()
            .and_then(|r| r.function(definition))
            .ok_or("exact import")?;
        assert_eq!(record.linkage, Linkage::Import);
        assert!(matches!(record.naming, Naming::Exact(_)));
        assert!(matches!(
            facts.types.kind(record.signature),
            Some(Kind::Function {
                convention: Convention::C,
                ..
            })
        ));
        let mut steps = 0;
        let original = crate::sema::crates::source_original(facts, definition, &mut steps)?
            .ok_or("actual retained original")?;
        assert_eq!(original.definition(), definition);
    }
    Ok(())
}
fn publish(
    artifact: &crates::Interface,
    rows: s::SourceRows,
    work: &mut g::Work,
    base: usize,
) -> std::result::Result<g::Payload, &'static str> {
    let descriptor = artifact.source_descriptor().ok_or("descriptor")?;
    let view = descriptor.rows();
    g::Payload::from_source_module_definitions(
        artifact.identity(),
        rows,
        view.header_attributes().ok_or("attributes")?.clone(),
        descriptor.module_contracts()?.rows().clone(),
        view.availability()?.clone(),
        base,
        work,
    )
}
#[test]
fn exact_rows_reject_foreign_missing_duplicate_and_legacy_ownership() -> Result {
    let (_, artifact) = provider(&fixture("provider")?)?;
    let (rows, attributes, modules) = test::raw(&artifact)?;
    let edits: [fn(&mut s::SourceRows); 8] = [
        |r| r.exact_callables[0].body.0 = u32::MAX,
        |r| r.exact_callables[0].declaration.0 = u32::MAX,
        |r| r.exact_callables[1].symbol = r.exact_callables[0].symbol.clone(),
        |r| r.exact_callables.swap(0, 1),
        |r| r.exact_callables[0].symbol = "".into(),
        |r| r.exact_callables[0].symbol = "x".repeat(4097).into(),
        |r| r.exact_callables[0].symbol = "nul\0symbol".into(),
        |r| {
            r.originals.remove(0);
        },
    ];
    for edit in edits {
        let mut changed = rows.clone();
        edit(&mut changed);
        let mut work = g::Work::default();
        assert!(publish(&artifact, changed, &mut work, 0).is_err());
        assert!(work.steps > 0);
    }
    let mut work = g::Work::default();
    assert!(publish(&artifact, rows.clone(), &mut work, crates::MAX_BYTES).is_err());
    assert!(work.steps > 0);
    let available = artifact
        .source_descriptor()
        .ok_or("descriptor")?
        .rows()
        .availability()?
        .clone();
    assert!(
        g::Payload::from_source_module_feature_predicates(
            artifact.identity(),
            rows,
            attributes,
            modules,
            available,
            0,
            &mut g::Work::default()
        )
        .is_err()
    );
    let mut bytes = crates::encode(&artifact)?;
    bytes[13] = 4;
    assert!(crates::decode(&bytes).is_err());
    for profile in [
        SourcePublication::LegacyFunctions,
        SourcePublication::FunctionContracts,
        SourcePublication::ModuleContracts,
        SourcePublication::ModuleAvailability,
        SourcePublication::ModuleFeaturePredicates,
        SourcePublication::ProvidedContracts,
    ] {
        assert!(test::exported_profile(&fixture("provider")?, profile, "exact_old").is_err());
    }
    Ok(())
}
#[test]
fn c_callable_safety_privacy_and_nominal_boundaries_remain_independent() -> Result {
    let source = fixture("provider")?;
    let (_, artifact) = provider(&source)?;
    let mismatch = "#![feature(no_core)]\n#![no_core]\nextern crate base;fn call(){let _:fn()->u8=base::byte;}";
    assert!(source_packets::client_analysis(artifact, mismatch).is_err());
    let private = source.replace("pub extern \"C\" fn byte", "extern \"C\" fn byte");
    let (program, artifact) = provider(&private)?;
    assert!(
        program
            .symbols
            .iter()
            .any(|s| s.name == "source_byte" && s.body.is_some())
    );
    assert!(
        source_packets::client_analysis(
            artifact,
            "#![feature(no_core)]\n#![no_core]\nextern crate base;fn call()->u8{base::byte()}"
        )
        .is_err()
    );
    let duplicate = source.replace("source_wide", "source_byte");
    assert!(provider(&duplicate).is_err());
    let nominal =
        crate::test_source::fixture("pa31/tests/spec/feature-bound-predicates/matching-caller.rs")?;
    let error = provider(&nominal)
        .err()
        .ok_or("original Value nominal7 unexpectedly admitted")?
        .to_string();
    assert!(
        error.contains("source-core provider requires no_core"),
        "{error}"
    );
    let nominal = format!(
        "{source}\n#[stable(feature=\"exact_nominal\",since=\"1.0.0\")]pub struct Value;\n"
    );
    let error = provider(&nominal)
        .err()
        .ok_or("nominal declaration unexpectedly admitted")?
        .to_string();
    assert!(
        error.contains("source declaration kind is outside"),
        "{error}"
    );
    assert_eq!(size_of::<s::ExactCallableExport>(), 24);
    assert_eq!(size_of::<s::SourceRows>(), 640);
    assert_eq!(size_of::<s::Columns>(), 616);
    assert_eq!(size_of::<crate::body::ir::Body>(), 456);
    assert_eq!(size_of::<crate::syntax::ast::Node>(), 36);
    Ok(())
}
