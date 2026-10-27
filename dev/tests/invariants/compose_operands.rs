//! Source raw constructors retain source declarations and ordinary pointer access proofs.
use super::*;
use crate::{
    pipeline,
    resolve::model::DefKind,
    sema::values::{Reader, View},
};
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
fn source(name: &str) -> TestResult<String> {
    let mut prefix =
        crate::test_source::fixture("pa31/tests/spec/pointer-composition/source-provider.rs")?;
    let input =
        crate::test_source::fixture(&format!("pa31/tests/spec/pointer-composition/{name}.rs"))?;
    for line in input
        .lines()
        .filter(|line| !line.starts_with("#!") && !line.starts_with("use core::"))
    {
        prefix.push_str(&line.replace(
            "core::intrinsics::aggregate_raw_ptr",
            "crate::intrinsics::aggregate_raw_ptr",
        ));
        prefix.push('\n');
    }
    Ok(prefix)
}
fn report(name: &str) -> TestResult<Report> {
    Ok(crate::test_source::source_provider(
        &source(name)?,
        "raw_parts_actual",
    )?)
}
#[test]
fn actual_source_unit_and_length_construction_does_not_read_pointee_memory() -> TestResult {
    for (name, expected) in [
        ("thin-read", 7),
        ("slice-metadata", 3),
        ("str-metadata", 3),
        ("null-metadata", 0),
        ("numeric-metadata", 99),
        ("bounds-metadata", 99),
        ("expired-metadata", 3),
        ("tail-metadata", 3),
        ("uninitialized-metadata", 1),
    ] {
        let report = report(name)?;
        let analysis = pipeline::analyze_source(&report).map_err(|e| format!("{name}: {e}"))?;
        let id = report
            .resolution
            .definitions
            .iter()
            .enumerate()
            .find_map(|(i, d)| {
                (d.kind == DefKind::Const && report.package.names.get(d.name) == "ANSWER")
                    .then_some(i)
            })
            .ok_or("actual result constant")?;
        let reader = Reader::new(&analysis.facts);
        let value = reader
            .definition(DefId(u32::try_from(id)?))
            .ok_or("completed constant")?;
        assert!(
            matches!(reader.view(value)?,View::Scalar(crate::sema::model::Scalar::Integer(n))if n==expected),
            "{name}"
        );
    }
    for name in ["foreign-output", "foreign-data-intrinsic"] {
        let report = report(name)?;
        pipeline::analyze_source(&report).map_err(|e| format!("{name}: {e}"))?;
    }
    Ok(())
}
#[test]
fn real_raw_slice_reference_consumer_reuses_existing_sequence_validity() -> TestResult {
    let mut text = source("slice-metadata")?;
    text=text.replace("const ANSWER:usize=ptr::metadata(ptr::slice_from_raw_parts(&DATA as *const [u8;3] as *const u8,3));", "const ANSWER:usize={let p=ptr::slice_from_raw_parts(&DATA as *const [u8;3] as *const u8,3);let s=unsafe{&*p};s[2] as usize};");
    let report = crate::test_source::source_provider(&text, "raw_parts_reference")?;
    pipeline::analyze_source(&report)?;
    let original =
        crate::test_source::fixture("pa31/tests/spec/pointer-composition/actual-slice-source.rs")?;
    assert!(original.contains("ub_checks::assert_unsafe_precondition!"));
    assert!(original.contains("&*ptr::slice_from_raw_parts(data, len)"));
    Ok(())
}
#[test]
fn bad_contracts_argument_faults_and_reification_do_not_gain_execution() -> TestResult {
    for name in [
        "wrong-metadata",
        "wide-data",
        "erased-unit-fault",
        "foreign-thin",
    ] {
        let input = source(name)?;
        let result = crate::test_source::source_provider(&input, "raw_parts_bad")
            .and_then(|r| pipeline::analyze_source(&r).map_err(|e| e.to_string()));
        assert!(result.is_err(), "{name}");
    }
    let text = source("slice-metadata")?;
    for mutated in [
        text.replace(
            "pub const fn aggregate_raw_ptr",
            "pub const unsafe fn aggregate_raw_ptr",
        ),
        text.replace("(data: D, meta: M) -> P", "(data: M, meta: D) -> P"),
    ] {
        let report = crate::test_source::source_provider(&mutated, "raw_parts_changed")?;
        assert!(pipeline::analyze_source(&report).is_err());
    }
    Ok(())
}
#[test]
fn actual_runtime_calls_keep_complete_typed_instances_and_cold_record_sizes() -> TestResult {
    let report = report("runtime-order")?;
    let pipeline::Analysis {
        mut facts,
        prepared,
    } = pipeline::analyze_source(&report)?;
    let result = crate::lower::instances::build_roots_prepared(
        &report,
        &mut facts,
        crate::lower::Roots {
            entry: None,
            exports: &[],
        },
        prepared,
    )?;
    assert!(!result.symbols.is_empty());
    let folder = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../obj/parallel/pa28/raw-pointer-composition-v1/candidate-runtime");
    std::fs::create_dir_all(&folder)?;
    let mut lowir = Vec::new();
    crate::lowir::writer::write(&result, &mut lowir)?;
    std::fs::write(folder.join("program.lowir"), lowir)?;
    std::fs::write(folder.join("source.rs"), source("runtime-order")?)?;

    assert_eq!(size_of::<SourceIntrinsic>(), 1);
    assert_eq!(size_of::<RawPartsSelection>(), 16);
    assert_eq!(size_of::<crate::body::ir::Body>(), 456);
    Ok(())
}

#[test]
fn associated_results_preserve_actual_sizing_roles_aliases_and_relaxation() -> TestResult {
    for name in [
        "pointee-parameter",
        "meta-parameter",
        "pointee-foreign",
        "default-sized",
        "explicit-sized-value",
        "relaxed-meta",
        "relaxed-explicit-pointee",
    ] {
        let text = crate::test_source::fixture(&format!(
            "pa31/tests/spec/pointer-composition/associated-sizing/{name}.rs"
        ))?;
        let report = crate::test_source::source_provider(&text, "associated_pointer_sizing")?;
        pipeline::analyze_source(&report).map_err(|e| format!("{name}: {e}"))?;
    }
    for name in [
        "default-unsized",
        "explicit-sized-foreign",
        "relaxed-pointee",
        "relaxed-foreign",
        "alias-pointee",
        "alias-meta",
        "shadow-pointee",
        "where-pointee",
    ] {
        let text = crate::test_source::fixture(&format!(
            "pa31/tests/spec/pointer-composition/associated-sizing/{name}.rs"
        ))?;
        let report = crate::test_source::source_provider(&text, "associated_pointer_sizing")?;
        assert!(pipeline::analyze_source(&report).is_err(), "{name}");
    }
    Ok(())
}

#[test]
fn raw_reference_opponents_keep_bounds_liveness_and_actual_read_duties() -> TestResult {
    for name in [
        "null-reference",
        "bounds-reference",
        "expired-reference",
        "uninitialized-read",
        "unaligned-reference",
        "shared-write",
    ] {
        let text = source(name)?
            .replace("slice::from_raw_parts(", "&*ptr::slice_from_raw_parts(")
            .replace(
                "slice::from_raw_parts_mut(",
                "&mut *ptr::slice_from_raw_parts_mut(",
            );
        let report = crate::test_source::source_provider(&text, "raw_parts_access")?;
        let error = pipeline::analyze_source(&report)
            .err()
            .ok_or_else(|| format!("{name}: invalid demanded access accepted"))?;
        println!("raw-reference-opponent={name} diagnostic={}", error.message);
        assert!(!error.message.contains("unresolved"), "{name}: {error}");
    }
    let mutable = source("mutable-reference")?.replace(
        "slice::from_raw_parts_mut(",
        "&mut *ptr::slice_from_raw_parts_mut(",
    );
    let report = crate::test_source::source_provider(&mutable, "raw_parts_mutable")?;
    pipeline::analyze_source(&report)?;
    let text = source("invalid-utf8-reference")?;
    let report = crate::test_source::source_provider(&text, "raw_parts_bytes")?;
    pipeline::analyze_source(&report)?;
    Ok(())
}
#[test]
fn actual_source_incarnation_instances_and_reified_targets_are_independent() -> TestResult {
    let text = source("slice-metadata")?;
    let report = crate::test_source::source_provider(&text, "raw_parts_identity")?;
    let pipeline::Analysis { facts, .. } = pipeline::analyze_source(&report)?;
    let catalog = facts.source_bindings.as_deref().ok_or("source catalog")?;
    let function = *facts
        .functions
        .iter()
        .find(|f| {
            report
                .package
                .names
                .get(report.resolution.definitions[f.definition.0 as usize].name)
                == "aggregate_raw_ptr"
        })
        .ok_or("actual intrinsic declaration")?;
    let body = facts
        .functions
        .iter()
        .find_map(|f| f.body)
        .ok_or("source body")?;
    let instance = (0..facts.types.len())
        .find_map(|i| {
            let ty = TypeId(u32::try_from(i).ok()?);
            let Some(Kind::FunctionItem {
                item, signature, ..
            }) = facts.types.kind(ty)
            else {
                return None;
            };
            let Some(Kind::Function { result, .. }) = facts.types.kind(signature) else {
                return None;
            };
            (item.0 == function.definition.0
                && matches!(facts.types.kind(result), Some(Kind::Pointer { .. })))
            .then(|| Instance::new(&facts.types, ty).ok())
            .flatten()
        })
        .ok_or("complete intrinsic instance")?;
    let mut selected = catalog
        .raw_parts_selection(&facts, instance, BodyOrigin::Source(body))?
        .ok_or("actual selection")?;
    catalog.require_raw_parts_report(&report, &facts, &selected)?;
    let other = crate::test_source::source_provider(&text, "raw_parts_identity")?;
    assert!(
        catalog
            .require_raw_parts_report(&other, &facts, &selected)
            .is_err()
    );
    selected.output = TypeId::UNIT;
    assert!(selected.current(&facts).is_err());
    let reified = text
        + "pub fn forbidden(){let _p:fn(*const u8,usize)->*const [u8]=intrinsics::aggregate_raw_ptr;}";
    let report = crate::test_source::source_provider(&reified, "raw_parts_reify")?;
    let error = pipeline::analyze_source(&report)
        .err()
        .ok_or("intrinsic reification gained execution")?;
    assert!(error.message.contains("function pointer"), "{error}");
    Ok(())
}

#[test]
fn repeated_construction_shares_actual_call_code_with_bounded_charged_work() -> TestResult {
    use std::fmt::Write as _;
    let basis =
        crate::test_source::fixture("pa31/tests/spec/pointer-composition/source-provider.rs")?;
    for width in [1usize, 8, 32] {
        let mut text = basis.clone();
        text.push_str("static DATA:[u8;3]=[7,8,9];");
        for i in 0..width {
            write!(
                text,
                "const C{i}:usize=ptr::metadata(ptr::slice_from_raw_parts(&DATA as *const [u8;3] as *const u8,3));"
            )?;
        }
        let report = crate::test_source::source_provider(&text, "raw_parts_scaling")?;
        let pipeline::Analysis { facts, .. } = pipeline::analyze_source(&report)?;
        let work = &facts.constant_work;
        println!(
            "raw-parts width={width} steps={} calls={} peak-slots={} code-bytes={} template-bytes={} source-ledger={:?}",
            work.steps,
            work.frames_started,
            work.peak_slots,
            work.code_bytes,
            work.template_bytes,
            crate::sema::header_attributes::attribute_storage(&facts)
        );
        assert!(work.steps < 100_000 + 20_000 * width);
        assert!(work.frames_started <= 32 + 10 * width);
        assert!(work.code_bytes < 1_000_000 + 100_000 * width);
    }
    let ordinary = crate::test_source::resolve("fn ordinary(x:usize)->usize{x}fn main(){}", true)?;
    let analysis = pipeline::analyze_source(&ordinary)?;
    assert!(analysis.facts.source_bindings.is_none());
    assert_eq!(size_of::<crate::types::Kind>(), 24);
    assert_eq!(size_of::<crate::syntax::ast::Node>(), 36);
    assert_eq!(size_of::<crate::sema::model::Operation>(), 16);
    Ok(())
}

#[test]
fn actual_pointer_metadata_uses_existing_abi_owner_without_pointee_reads() -> TestResult {
    let mut text = source("slice-metadata")?;
    text.push_str("trait Object{}type ObjectPointer=*const dyn Object;");
    let report = crate::test_source::source_provider(&text, "raw_parts_metadata_kind")?;
    let pipeline::Analysis { mut facts, .. } = pipeline::analyze_source(&report)?;
    let id = report
        .resolution
        .definitions
        .iter()
        .enumerate()
        .find_map(|(i, d)| {
            (d.kind == DefKind::TypeAlias && report.package.names.get(d.name) == "ObjectPointer")
                .then_some(i)
        })
        .ok_or("actual pointer alias")?;
    let ty = facts.definitions[id].ok_or("actual pointer type")?;
    let mut evaluator = crate::sema::values::Evaluator::new(&report, &mut facts);
    assert!(
        evaluator
            .require_raw_parts_metadata(ty, TypeId::UNIT, 32)
            .is_err()
    );
    assert!(
        evaluator
            .require_raw_parts_metadata(ty, TypeId::primitive(crate::types::Primitive::Usize), 32)
            .is_err()
    );
    assert!(
        evaluator
            .require_raw_parts_metadata(ty, TypeId::UNIT, 0)
            .is_err()
    );
    Ok(())
}
