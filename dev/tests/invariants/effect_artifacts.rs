//! Written callable effects survive a fresh artifact without granting body proof.
use crate::{
    lower::instances::SourcePublication,
    module_packets, source_packets,
    support::{
        constness::Constness,
        crates::{
            self,
            generics::{self as g, source as s},
        },
    },
};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const BASE: &str = "pa31/tests/spec/source-callable-effects/";
fn fixture(name: &str) -> Result<String> {
    Ok(crate::test_source::fixture(&format!("{BASE}{name}.rs"))?)
}
fn provider(name: &str) -> Result<crates::Interface> {
    module_packets::exported_profile(
        &fixture(name)?,
        SourcePublication::ModuleCallables,
        "callable_effects",
    )
}
fn bytes() -> Result<Vec<u8>> {
    Ok(crates::encode(&provider("combined-provider")?)?)
}
fn stage(
    artifact: &crates::Interface,
    rows: s::SourceRows,
    base: usize,
    work: &mut g::Work,
) -> Result<g::Payload> {
    let (_, attributes, modules) = module_packets::raw(artifact)?;
    Ok(g::Payload::from_source_module_callables(
        artifact.identity(),
        rows,
        attributes,
        modules,
        artifact
            .source_descriptor()
            .ok_or("actual descriptor")?
            .availability()?
            .clone(),
        base,
        work,
    )?)
}

#[test]
fn full_provider_and_generic_const_originals_execute_in_a_fresh_store() -> Result {
    let bytes = bytes()?;
    assert_eq!(bytes.get(13..15), Some([7, 0].as_slice()));
    assert_eq!(crates::encode(&crates::decode(&bytes)?)?, bytes);
    let (
        report,
        crate::pipeline::Analysis {
            mut facts,
            prepared,
        },
        _,
    ) = source_packets::client_analysis(crates::decode(&bytes)?, &fixture("native-client")?)?;
    assert_eq!(
        facts
            .warnings()
            .filter(|w| w.code == crate::support::diagnostics::WarningCode::Deprecated)
            .count(),
        2
    );
    let program = crate::lower::instances::build_roots_prepared(
        &report,
        &mut facts,
        crate::lower::Roots {
            entry: None,
            exports: &[],
        },
        prepared,
    )?;
    let folder = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../obj/parallel/source-callable-effects-v1/candidate-v1");
    std::fs::create_dir_all(&folder)?;
    std::fs::write(folder.join("provider.rgcrate"), bytes)?;
    crate::lowir::writer::write(
        &program,
        &mut std::fs::File::create(folder.join("client.lowir"))?,
    )?;
    Ok(())
}

#[test]
fn client_const_api_unsafe_and_function_pointer_checks_remain_independent() -> Result {
    let bytes = bytes()?;
    source_packets::client_analysis(crates::decode(&bytes)?, &fixture("unsafe-fnptr-allowed")?)?;
    for name in [
        "nonconst-denied",
        "unsafe-required",
        "const-unsafe-required",
        "unsafe-fnptr-denied",
        "fnptr-const-denied",
    ] {
        assert!(
            source_packets::client_analysis(crates::decode(&bytes)?, &fixture(name)?).is_err(),
            "{name}"
        );
    }
    let missing =
        crate::test_source::fixture("pa31/tests/spec/source-module-compatibility/const-denied.rs")?;
    assert!(source_packets::client_analysis(crates::decode(&bytes)?, &missing).is_err());
    for name in ["nonconst-provider-denied", "internal-unsafe-required"] {
        assert!(provider(name).is_err(), "{name}");
    }
    Ok(())
}

#[test]
fn older_profiles_keep_effect_denials_and_cannot_decode_a_downgraded_body() -> Result {
    let source = fixture("provider")?;
    for profile in [
        SourcePublication::LegacyFunctions,
        SourcePublication::FunctionContracts,
        SourcePublication::ModuleContracts,
        SourcePublication::ModuleAvailability,
        SourcePublication::ModuleFeaturePredicates,
        SourcePublication::ModuleDefinitions,
        SourcePublication::ModuleCompatibility,
    ] {
        assert!(
            module_packets::exported_profile(&source, profile, "old_callables").is_err(),
            "{profile:?}"
        );
    }
    let bytes = bytes()?;
    for version in 0u16..7 {
        let mut older = bytes.clone();
        older[13..15].copy_from_slice(&version.to_le_bytes());
        assert!(crates::decode(&older).is_err(), "version{version}");
    }
    Ok(())
}

#[test]
fn defining_const_context_and_actual_safety_cannot_be_mutated_or_unpaid() -> Result {
    let artifact = provider("provider")?;
    let (original, _, _) = module_packets::raw(&artifact)?;
    let constant = original
        .functions
        .iter()
        .position(|f| f.constness == Constness::Const)
        .ok_or("actual const function")?;
    let unsafe_ = original
        .functions
        .iter()
        .position(|f| f.original.unsafe_)
        .ok_or("actual unsafe function")?;
    for mutation in 0..6 {
        let mut rows = original.clone();
        let function = rows.functions[constant];
        let s::FunctionBody::Original(body) = function.body else {
            return Err("actual const body".into());
        };
        match mutation {
            0 => rows.originals[body.0 as usize].context = s::CallContext::Runtime,
            1 => rows.functions[constant].constness = Constness::Conditional,
            2 => rows.functions[constant].body = s::FunctionBody::Absent,
            3 => rows.functions[unsafe_].callable.unsafe_ = false,
            4 => rows.functions[constant].declaration.0 = u32::MAX,
            _ => rows.body_owners[body.0 as usize].declaration.0 = u32::MAX,
        }
        let mut work = g::Work::default();
        assert!(
            stage(&artifact, rows, 0, &mut work).is_err(),
            "mutation{mutation}"
        );
        assert!(work.steps > 0);
    }
    let mut work = g::Work {
        steps: crates::MAX_COMPONENTS,
        ..g::Work::default()
    };
    assert!(stage(&artifact, original.clone(), 0, &mut work).is_err());
    assert!(work.steps > crates::MAX_COMPONENTS);
    assert!(
        stage(
            &artifact,
            original,
            crates::MAX_BYTES,
            &mut g::Work::default()
        )
        .is_err()
    );
    Ok(())
}

#[test]
fn actual_const_marker_declarations_keep_owner_flags_and_supertrait_identity() -> Result {
    let artifact = provider("combined-provider")?;
    let (original, _, _) = module_packets::raw(&artifact)?;
    let destruct = original
        .roles
        .iter()
        .find(|r| r.role == crate::support::bindings::LangItem::Destruct)
        .ok_or("actual Destruct role")?
        .declaration
        .declaration;
    let index = original
        .traits
        .iter()
        .position(|h| h.declaration == destruct)
        .ok_or("actual Destruct header")?;
    for mutation in 0..4 {
        let mut rows = original.clone();
        match mutation {
            0 => rows.traits[index].flags = s::TraitFlags::from_bits(0)?,
            1 => {
                rows.traits[index].written_supertraits.end =
                    rows.traits[index].written_supertraits.start
            }
            2 => rows.traits[index].self_type.0 = u32::MAX,
            _ => {
                let role = rows
                    .roles
                    .iter_mut()
                    .find(|r| r.role == crate::support::bindings::LangItem::Freeze)
                    .ok_or("actual Freeze role")?;
                role.declaration.artifact = s::ArtifactRef(1);
            }
        }
        assert!(
            stage(&artifact, rows, 0, &mut g::Work::default()).is_err(),
            "mutation{mutation}"
        );
    }
    Ok(())
}

#[test]
fn reconstructed_const_mode_is_revalidated_without_claiming_a_checked_body() -> Result {
    let (report, analysis, _) = source_packets::client_analysis(
        crates::decode(&bytes()?)?,
        &fixture("unsafe-fnptr-allowed")?,
    )?;
    let definition = report
        .resolution
        .definitions
        .iter()
        .enumerate()
        .find(|(_, d)| d.node.is_none() && report.package.names.get(d.name) == "const_marked")
        .map(|(index, _)| u32::try_from(index).map(crate::resolve::model::DefId))
        .transpose()?
        .ok_or("installed canonical const declaration")?;
    let mut work = super::ImportedOriginals::default();
    let view = crate::sema::crates::source_original(&analysis.facts, definition, &mut work.steps)?
        .ok_or("actual source original")?;
    let mut body = super::build(&view, &mut work)?;
    assert!(body.const_context);
    super::validate(&view, &body, &mut work.steps)?;
    body.const_context = false;
    assert!(super::validate(&view, &body, &mut work.steps).is_err());
    Ok(())
}

#[test]
fn independently_added_const_bodies_keep_linear_validation_work_and_hot_widths() -> Result {
    let mut previous = None;
    for width in [8, 32, 128] {
        let mut source = fixture("provider")?;
        for index in 0..width {
            source.push_str(&format!("\n#[stable(feature=\"callable_batch\",since=\"1.0.0\")]#[rustc_const_stable(feature=\"callable_batch\",since=\"1.0.0\")]pub const fn added_{index}()->u8{{7}}\n"));
        }
        let artifact = module_packets::exported_profile(
            &source,
            SourcePublication::ModuleCallables,
            "callable_batch",
        )?;
        let (rows, _, _) = module_packets::raw(&artifact)?;
        let mut work = g::Work::default();
        let payload = stage(&artifact, rows, 0, &mut work)?;
        let mut repeated = g::Work::default();
        let (rows, _, _) = module_packets::raw(&artifact)?;
        stage(&artifact, rows, 0, &mut repeated)?;
        assert_eq!(work, repeated);
        if let Some((old_width, old_work, old_bytes)) = previous {
            assert!(work.steps <= old_work * width / old_width + 512);
            assert!(payload.retained_bytes() <= old_bytes * width / old_width + 4096);
        }
        eprintln!(
            "callable width={width} work={} retained={}",
            work.steps,
            payload.retained_bytes()
        );
        previous = Some((width, work.steps, payload.retained_bytes()));
    }
    assert_eq!(size_of::<crate::body::ir::Body>(), 456);
    assert_eq!(size_of::<crate::syntax::ast::Node>(), 36);
    println!(
        "FunctionHeader={} SourceOriginal={} Body={}",
        size_of::<s::FunctionHeader>(),
        size_of::<s::Original>(),
        size_of::<crate::body::ir::Body>()
    );
    Ok(())
}
