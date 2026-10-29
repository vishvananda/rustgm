//! Real lexical obligations survive source publication without importing paid proof.
use crate::{
    lower::instances::SourcePublication,
    module_packets, source_packets,
    support::crates::{
        self,
        generics::{self as g, source as s},
    },
};
use std::fmt::Write as _;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
fn fixture(name: &str) -> Result<String> {
    Ok(crate::test_source::fixture(&format!(
        "pa31/tests/spec/source-call-safety/{name}.rs"
    ))?)
}
fn provider() -> Result<crates::Interface> {
    module_packets::exported_profile(
        &fixture("provider")?,
        SourcePublication::ModuleSafety,
        "call_safety",
    )
}
fn stage(
    artifact: &crates::Interface,
    rows: s::SourceRows,
    work: &mut g::Work,
) -> Result<g::Payload> {
    let (_, attributes, modules) = module_packets::raw(artifact)?;
    Ok(g::Payload::from_source_module_safety(
        artifact.identity(),
        rows,
        attributes,
        modules,
        artifact
            .source_descriptor()
            .ok_or("actual descriptor")?
            .availability()?
            .clone(),
        0,
        work,
    )?)
}
#[test]
fn original_forwarder_and_actual_call_contexts_reach_a_fresh_native_producer() -> Result {
    let artifact = provider()?;
    let bytes = crates::encode(&artifact)?;
    assert_eq!(bytes.get(13..15), Some([8, 0].as_slice()));
    assert_eq!(crates::encode(&crates::decode(&bytes)?)?, bytes);
    let rows = artifact
        .source_descriptor()
        .ok_or("actual descriptor")?
        .rows();
    for kind in [
        s::CallSafety::Safe,
        s::CallSafety::ExplicitUnsafeBlock,
        s::CallSafety::UnsafeFunction,
    ] {
        assert!(
            rows.originals
                .iter()
                .flat_map(|body| &body.calls)
                .any(|call| call.safety == kind)
        );
    }
    let program = source_packets::client(crates::decode(&bytes)?, &fixture("native-client")?)?;
    let folder = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../obj/parallel/source-callable-effects-v1/safety-candidate-v1");
    std::fs::create_dir_all(&folder)?;
    std::fs::write(folder.join("provider.rgcrate"), bytes)?;
    crate::lowir::writer::write(
        &program,
        &mut std::fs::File::create(folder.join("client.lowir"))?,
    )?;
    assert!(source_packets::client_analysis(provider()?, &fixture("missing-client")?).is_err());
    Ok(())
}
#[test]
fn old_profiles_and_downgraded_packets_retain_their_unsafe_body_fences() -> Result {
    for profile in [
        SourcePublication::LegacyFunctions,
        SourcePublication::FunctionContracts,
        SourcePublication::ModuleContracts,
        SourcePublication::ModuleAvailability,
        SourcePublication::ModuleFeaturePredicates,
        SourcePublication::ModuleDefinitions,
        SourcePublication::ModuleCompatibility,
        SourcePublication::ModuleCallables,
    ] {
        assert!(
            module_packets::exported_profile(
                &fixture("preserved-provider")?,
                profile,
                "old_safety"
            )
            .is_err(),
            "{profile:?}"
        );
    }
    let bytes = crates::encode(&provider()?)?;
    for version in 0u16..8 {
        let mut old = bytes.clone();
        old[13..15].copy_from_slice(&version.to_le_bytes());
        assert!(crates::decode(&old).is_err(), "version {version}");
    }
    Ok(())
}
fn missing_or_forged(rows: &mut s::SourceRows, mutation: usize) -> Result {
    let safe_caller =
        rows.originals
            .iter()
            .position(|body| {
                rows.functions.iter().any(|header| {
                    header.declaration == body.declaration && !header.callable.unsafe_
                }) && body
                    .calls
                    .iter()
                    .any(|call| call.safety == s::CallSafety::ExplicitUnsafeBlock)
            })
            .ok_or("actual safe caller with unsafe obligation")?;
    let request = rows.originals[safe_caller]
        .calls
        .iter_mut()
        .find(|call| call.safety == s::CallSafety::ExplicitUnsafeBlock)
        .ok_or("actual discharged call")?;
    match mutation {
        0 => request.safety = s::CallSafety::Safe,
        1 => request.safety = s::CallSafety::UnsafeFunction,
        2 => request.site = u32::MAX,
        3 => request.signature = g::TypeRef(u32::MAX),
        4 => request.value = u32::MAX,
        5 => {
            rows.function_applications[request.target.0 as usize]
                .declaration
                .artifact = s::ArtifactRef(1)
        }
        _ => {
            let safe = rows
                .originals
                .iter_mut()
                .flat_map(|body| &mut body.calls)
                .find(|call| call.safety == s::CallSafety::Safe)
                .ok_or("actual safe call")?;
            safe.safety = s::CallSafety::ExplicitUnsafeBlock;
        }
    }
    Ok(())
}
#[test]
fn missing_forged_foreign_and_unpaid_call_contexts_cannot_publish() -> Result {
    let artifact = provider()?;
    for mutation in 0..7 {
        let (mut rows, _, _) = module_packets::raw(&artifact)?;
        missing_or_forged(&mut rows, mutation)?;
        assert!(
            stage(&artifact, rows, &mut g::Work::default()).is_err(),
            "mutation {mutation}"
        );
    }
    let (rows, _, _) = module_packets::raw(&artifact)?;
    let mut failed = g::Work {
        steps: crates::MAX_COMPONENTS,
        ..g::Work::default()
    };
    assert!(stage(&artifact, rows, &mut failed).is_err());
    assert!(failed.steps > crates::MAX_COMPONENTS);
    let (rows, _, _) = module_packets::raw(&artifact)?;
    assert!(stage(&artifact, rows, &mut failed).is_err());
    let (rows, _, _) = module_packets::raw(&artifact)?;
    stage(&artifact, rows, &mut g::Work::default())?;
    Ok(())
}
#[test]
fn call_context_work_and_retention_scale_with_touched_source_calls() -> Result {
    let mut measured = Vec::new();
    for width in [8, 32, 128] {
        let mut source = fixture("provider")?;
        for index in 0..width {
            writeln!(
                source,
                "#[stable(feature=\"call_safety\",since=\"1.0.0\")]pub fn added_{index}(value:u8)->u8{{unsafe{{identity(value)}}}}"
            )?;
        }
        let artifact = module_packets::exported_profile(
            &source,
            SourcePublication::ModuleSafety,
            "safety_scaling",
        )?;
        let (rows, _, _) = module_packets::raw(&artifact)?;
        let mut work = g::Work::default();
        let payload = stage(&artifact, rows, &mut work)?;
        let (rows, _, _) = module_packets::raw(&artifact)?;
        let mut warm = g::Work::default();
        stage(&artifact, rows, &mut warm)?;
        assert_eq!(work, warm);
        eprintln!(
            "safety width={width} work={} retained={}",
            work.steps,
            payload.retained_bytes()
        );
        measured.push((work.steps, payload.retained_bytes()));
    }
    for column in 0..2 {
        let at = |index: usize| {
            if column == 0 {
                measured[index].0
            } else {
                measured[index].1
            }
        };
        let small = at(1) - at(0);
        let large = at(2) - at(1);
        assert!(small > 0 && large <= 4 * small + 4096);
    }
    assert_eq!(size_of::<s::CallRequest>(), 44);
    assert_eq!(size_of::<crate::body::ir::Body>(), 456);
    assert_eq!(size_of::<crate::syntax::ast::Node>(), 36);
    Ok(())
}
