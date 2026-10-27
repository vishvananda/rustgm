//! Real source caller data is separate from callable proof and source signatures.
use crate::{
    body,
    const_eval::code::Code,
    sema, test_source,
    types::{TypeId, instance::Instance},
};
type TestResult = Result<(), Box<dyn std::error::Error>>;
const PROVIDER: &str = "pa31/tests/spec/source-location/caller-location.rs";

#[test]
fn direct_source_caller_locations_use_const_frames_and_runtime_hidden_abi() -> TestResult {
    let source = test_source::fixture(PROVIDER)?;
    let report = test_source::source_provider(&source, "caller_native")?;
    let crate::pipeline::Analysis {
        mut facts,
        prepared,
    } = crate::pipeline::analyze_source(&report)?;
    let owner = facts
        .source_bindings
        .as_deref()
        .and_then(|catalog| catalog.call_contexts.as_deref())
        .ok_or("actual caller context")?;
    println!(
        "caller context owner={} Header={} CallableInputs={} CallerSite={} Operand={} Code={} Body={} Rvalue={} Value={}",
        size_of_val(owner),
        size_of::<super::super::Header>(),
        size_of::<body::ir::ResolvedCallables>(),
        size_of::<body::ir::CallerSite>(),
        size_of::<crate::const_eval::code::PreparedOperand>(),
        size_of::<Code>(),
        size_of::<body::ir::Body>(),
        size_of::<body::ir::Rvalue>(),
        size_of::<body::ir::Value>()
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
    for name in [
        "caller_line",
        "caller_column",
        "constant_line",
        "constant_column",
        "filename_pointer",
        "filename_length",
    ] {
        assert!(
            program
                .symbols
                .iter()
                .any(|symbol| symbol.name == name && symbol.body.is_some()),
            "{name}"
        );
    }
    let out = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../obj/parallel/all-attribute-selected-output-v1/native/source-caller");
    std::fs::create_dir_all(&out)?;
    std::fs::write(out.join("provider.rs"), source)?;
    crate::lowir::writer::write(
        &program,
        &mut std::fs::File::create(out.join("provider.lowir"))?,
    )?;
    Ok(())
}

#[test]
fn caller_metadata_cannot_replace_targets_or_forge_snapshot_identity() -> TestResult {
    let report = test_source::resolve("const fn f()->u32{7}const fn g()->u32{f()}", true)?;
    let facts = sema::declarations::analyze(&report)?;
    let mut facts = sema::bodies::analyze(&report, facts)?;
    let mut program = body::build(&report, &facts)?;
    let actual = program
        .bodies
        .iter_mut()
        .find(|body| {
            body.values
                .iter()
                .any(|value| matches!(value.kind, body::ir::Rvalue::Call { .. }))
        })
        .ok_or("actual ordinary call body")?;
    assert!(!actual.has_resolved_callables());
    let (index, callee) = actual
        .values
        .iter()
        .enumerate()
        .find_map(|(index, value)| {
            if let body::ir::Rvalue::Call { callee, .. } = value.kind {
                Some((index, callee))
            } else {
                None
            }
        })
        .ok_or("call")?;
    let callable = actual.values[callee.0 as usize].ty;
    let location = sema::values::Evaluator::new(&report, &mut facts)
        .borrow(
            sema::values::Constant::scalar(sema::model::Scalar::Integer(1)),
            TypeId::primitive(crate::types::Primitive::U32),
        )?
        .location()
        .ok_or("real test allocation")?;
    let site = body::ir::CallerSite {
        owner: actual.source.source().ok_or("source owner")?,
        value: body::ir::ValueId(u32::try_from(index)?),
        callable,
        location,
    };
    actual.append_caller_site(site, &facts)?;
    actual.seal_caller_sites(&facts)?;
    assert!(!actual.has_resolved_callables());
    let instance = Instance::new(&facts.types, callable)?;
    actual.install_callable_targets(crate::support::hash::HashMap::from_iter([(
        instance, instance,
    )]))?;
    assert!(actual.has_resolved_callables());
    assert_eq!(actual.caller_site(site.value)?, Some(site));
    let snapshot = Code::snapshot(actual)?;
    assert_eq!(snapshot.caller_site(site.value)?, Some(site));
    assert_eq!(snapshot.operands.len(), 1);
    actual.append_caller_site(site, &facts)?;
    assert!(actual.seal_caller_sites(&facts).is_err());
    assert!(Code::snapshot(actual).is_err());
    assert_eq!(size_of::<body::ir::Body>(), 448);
    assert_eq!(size_of::<body::ir::Rvalue>(), 20);
    assert_eq!(size_of::<body::ir::Value>(), 24);
    assert_eq!(size_of::<body::ir::ResolvedCallables>(), 56);
    assert_eq!(size_of::<super::super::Header>(), 72);
    assert_eq!(size_of::<crate::const_eval::code::PreparedOperand>(), 28);
    assert_eq!(size_of::<Code>(), 304);
    Ok(())
}

#[test]
fn intrinsic_const_policy_and_track_caller_opponents_remain_actual_source_checks() -> TestResult {
    let source = test_source::fixture(PROVIDER)?;
    for (label, changed, expected) in [
        (
            "feature",
            source.replace("feature(core_intrinsics,", "feature("),
            "source intrinsic const API requires",
        ),
        (
            "marker",
            source.replace("#[rustc_intrinsic_const_stable_indirect]", ""),
            "const-stable body cannot expose",
        ),
        (
            "shape",
            source.replace(
                "pub const fn caller_location()",
                "pub const fn caller_location(value:u8)",
            ),
            "caller_location",
        ),
    ] {
        let outcome =
            test_source::source_provider(&changed, "caller_negative").and_then(|report| {
                crate::pipeline::analyze_source(&report)
                    .map(|_| ())
                    .map_err(|error| error.to_string())
            });
        let error = outcome
            .err()
            .ok_or_else(|| format!("{label} was accepted"))?;
        assert!(error.to_string().contains(expected), "{label}: {error}");
    }
    Ok(())
}
