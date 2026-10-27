//! Real nominal array fields feed SIMD layout and the existing object boundary.
use crate::{
    abi::{
        classification::{Class, Classes, Passing, Plans},
        layout::{Layouts, Shape},
    },
    pipeline::{self, Analysis},
    resolve::{
        expansion::Report,
        model::{DefId, DefKind},
    },
    types::{
        Kind, TypeId,
        aggregate::{Definitions, SimdShape},
    },
};
use std::fmt::Write as _;
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
fn analyze(source: &str) -> TestResult<(Report, Analysis)> {
    let report = crate::test_source::resolve(source, true)?;
    let analysis = pipeline::analyze_source(&report)?;
    Ok((report, analysis))
}
fn nominal(
    report: &Report,
    facts: &crate::sema::model::Declarations,
    name: &str,
) -> TestResult<TypeId> {
    let index = report
        .resolution
        .definitions
        .iter()
        .position(|d| d.kind == DefKind::Struct && report.package.names.get(d.name) == name)
        .ok_or("actual nominal")?;
    Ok(facts.definitions[index].ok_or("nominal type")?)
}
fn input(name: &str) -> TestResult<String> {
    Ok(crate::test_source::fixture(&format!(
        "pa41/tests/spec/source-simd/{name}.rs"
    ))?)
}
#[test]
fn actual_declarations_gates_and_captured_attributes_keep_pin_authority() -> TestResult {
    for name in [
        "cfg-disabled",
        "cfg-attribute-disabled",
        "gate-defined-literal",
        "gate-captured-meta",
        "gate-captured-hint",
        "generic",
        "field-access",
    ] {
        analyze(&input(name)?)?;
    }
    for (name, message) in [
        (
            "missing-repr-gate",
            "SIMD representation requires repr_simd",
        ),
        (
            "empty",
            "SIMD representation requires exactly one array field",
        ),
        (
            "multiple",
            "SIMD representation requires exactly one array field",
        ),
        ("nonarray", "SIMD representation requires an array field"),
        (
            "bool",
            "SIMD declaration lane must be an integer, float, raw pointer or parameter",
        ),
        ("zero", "SIMD lane count must be between 1 and 32768"),
    ] {
        let report = crate::test_source::resolve(&input(name)?, true)?;
        let error = pipeline::analyze_source(&report)
            .err()
            .ok_or("expected representation rejection")?;
        assert_eq!(error.message, message, "{name}");
    }
    Ok(())
}
#[test]
fn lane_counts_padding_alignment_and_generic_scalar_storage_are_actual() -> TestResult {
    let text = "#![feature(repr_simd)] #[repr(simd)]struct I([i64;2]);#[repr(simd)]struct F([f32;4]);#[repr(simd)]struct D([f64;2]);#[repr(simd)]struct Odd([f32;3]);#[repr(simd,packed)]struct Packed([f32;3]);#[repr(simd,align(8))]struct Aligned([f32;4]);fn main(){}";
    let (report, Analysis { facts, .. }) = analyze(text)?;
    for (name, size, align) in [
        ("I", 16, 16),
        ("F", 16, 16),
        ("D", 16, 16),
        ("Odd", 16, 16),
        ("Packed", 12, 4),
        ("Aligned", 16, 16),
    ] {
        let ty = nominal(&report, &facts, name)?;
        let layout = facts.layouts.get(ty).ok_or("actual SIMD layout")?;
        assert_eq!((layout.size, layout.align), (size, align), "{name}");
        let [variant] = facts.layouts.variants(layout) else {
            return Err("one actual variant".into());
        };
        assert_eq!(facts.layouts.offsets(variant), [0]);
        assert!(matches!(layout.shape, Shape::Aggregate { tag: None, .. }));
        assert!(matches!(
            facts.aggregates.simd_shape(&facts.types, ty)?,
            SimdShape::Ready { .. }
        ));
    }
    for (lane, accepted) in [("u8", true), ("bool", true), ("Lane", false)] {
        let source = format!(
            "#![feature(repr_simd)]#[repr(simd)]struct Generic<T>([T;4]);struct Lane(u8,u8);fn demand(x:Generic<{lane}>)->Generic<{lane}>{{x}}fn main(){{}}"
        );
        let (_, Analysis { mut facts, .. }) = analyze(&source)?;
        let function = facts.functions.iter().find(|function| matches!(facts.types.kind(function.signature),Some(Kind::Function {result,..}) if matches!(facts.types.kind(result),Some(Kind::Adt(_, _))))).ok_or("actual generic use")?;
        let Some(Kind::Function { result, .. }) = facts.types.kind(function.signature) else {
            return Err("actual signature".into());
        };
        let answer = crate::abi::templates::Templates::default().demand(
            &mut facts.types,
            &mut facts.aggregates,
            &mut facts.layouts,
            result,
        );
        if accepted {
            assert!(answer?, "actual scalar lane layout");
        } else {
            assert_eq!(
                answer.err().ok_or("product lane must reject")?.message,
                "SIMD lane does not have a primitive scalar representation"
            );
        }
    }
    Ok(())
}
#[test]
fn foreign_direct_gate_does_not_strengthen_definitions_or_nested_wrappers() -> TestResult {
    for name in ["foreign-ffi", "definition-no-ffi", "nested-foreign-no-ffi"] {
        analyze(&input(name)?)?;
    }
    let report = crate::test_source::resolve(&input("foreign-missing-ffi")?, true)?;
    assert_eq!(
        pipeline::analyze_source(&report)
            .err()
            .ok_or("actual direct FFI gate")?
            .message,
        "foreign SIMD declaration requires simd_ffi"
    );
    Ok(())
}
#[test]
fn c_vectors_use_sseup_and_large_nested_vectors_cannot_hide_in_memory() -> TestResult {
    let (report, Analysis { mut facts, .. }) = analyze(
        "#![feature(repr_simd)]#[repr(simd)]struct V([f32;4]);#[repr(C)]struct W{v:V}extern \"C\" fn f(x:W)->W{x}fn main(){}",
    )?;
    let ty = nominal(&report, &facts, "W")?;
    let mut plans = Plans::default();
    let passing = plans.passing(&facts.types, &facts.aggregates, &mut facts.layouts, ty)?;
    assert!(
        matches!(passing, Passing::Object(object) if object.classes == Classes::Eightbytes { first: Class::Sse, second: Some(Class::SseUp) })
    );
    let (report, Analysis { mut facts, .. }) = analyze(
        "#![feature(repr_simd)]#[repr(simd)]struct V([f32;8]);#[repr(C)]struct W{v:V}extern \"C\" fn f(x:W)->W{x}fn main(){}",
    )?;
    for name in ["V", "W"] {
        let ty = nominal(&report, &facts, name)?;
        assert_eq!(
            Plans::default()
                .passing(&facts.types, &facts.aggregates, &mut facts.layouts, ty)
                .err(),
            Some("C SIMD boundary supports only aligned 128-bit vectors")
        );
    }
    let span = crate::lowir::program::MemorySpan {
        bytes: 16,
        align: 16,
    };
    let classes = crate::lowir::boundary::Classes::Eightbytes {
        first: crate::lowir::boundary::Class::Integer,
        second: Some(crate::lowir::boundary::Class::SseUp),
    };
    assert!(
        crate::lowir::boundary::Object { span, classes }
            .validate(false)
            .is_err()
    );
    Ok(())
}
#[test]
fn genuine_source_rust_and_c_copy_bodies_lower_with_distinct_signatures() -> TestResult {
    let report =
        crate::test_source::source_provider(&input("native-family-corrected")?, "source_simd")?;
    let Analysis {
        mut facts,
        prepared,
    } = pipeline::analyze_source(&report)?;
    let roots = ["echo_f", "echo_d", "echo_i", "rust_bridge"]
        .iter()
        .map(|name| {
            report
                .resolution
                .definitions
                .iter()
                .enumerate()
                .find(|(_, d)| {
                    d.kind == DefKind::Function && report.package.names.get(d.name) == *name
                })
                .ok_or("actual exported function")
                .and_then(|(index, _)| {
                    u32::try_from(index)
                        .map(DefId)
                        .map_err(|_| "function identity")
                })
        })
        .collect::<Result<Vec<_>, _>>()?;
    let program = crate::lower::instances::build_roots_prepared(
        &report,
        &mut facts,
        crate::lower::Roots {
            entry: None,
            exports: &roots,
        },
        prepared,
    )?;
    let mut bytes = Vec::new();
    crate::lowir::writer::write(&program, &mut bytes)?;
    let text = String::from_utf8(bytes)?;
    assert!(text.contains("sse_sseup"), "{text}");
    assert!(
        text.contains("rust_identity"),
        "actual Rust identity body is retained"
    );
    Ok(())
}
#[test]
fn shared_shape_queries_warm_abi_and_ordinary_paths_keep_bounded_storage() -> TestResult {
    for count in [1, 8, 32, 128] {
        let mut source = String::from("#![feature(repr_simd)]");
        for index in 0..count {
            write!(source, "#[repr(simd)]struct V{index}([f32;4]);")?;
        }
        source.push_str("fn main(){}");
        let (report, Analysis { mut facts, .. }) = analyze(&source)?;
        let mut plans = Plans::default();
        for index in 0..count {
            let ty = nominal(&report, &facts, &format!("V{index}"))?;
            let first = plans.passing(&facts.types, &facts.aggregates, &mut facts.layouts, ty)?;
            let retained = plans.retained_bytes;
            for _ in 0..8 {
                assert_eq!(
                    plans.passing(&facts.types, &facts.aggregates, &mut facts.layouts, ty)?,
                    first
                );
            }
            assert_eq!(plans.retained_bytes, retained);
        }
        eprintln!(
            "simd-count={count} abi-steps={} abi-retained={} layout-steps={} layout-records={} offsets={} variants={}",
            plans.steps,
            plans.retained_bytes,
            facts.layouts.steps,
            facts.layouts.records.capacity(),
            facts.layouts.offsets.capacity(),
            facts.layouts.variants.capacity()
        );
    }
    let (report, Analysis { facts, .. }) = analyze("struct Ordinary(u64);fn main(){}")?;
    let ty = nominal(&report, &facts, "Ordinary")?;
    assert_eq!(
        facts.aggregates.simd_shape(&facts.types, ty)?,
        SimdShape::NotSimd
    );
    for _ in 0..128 {
        assert_eq!(
            facts.aggregates.simd_shape(&facts.types, TypeId::UNIT)?,
            SimdShape::NotSimd
        );
    }
    eprintln!(
        "SIMD enclosing: Repr={} Definition={} Definitions={} Layout={} Layouts={} TypeKind={} Body={} Report={}",
        size_of::<crate::types::aggregate::Repr>(),
        size_of::<crate::types::aggregate::Definition>(),
        size_of::<Definitions>(),
        size_of::<crate::abi::layout::Layout>(),
        size_of::<Layouts>(),
        size_of::<Kind>(),
        size_of::<crate::body::ir::Body>(),
        size_of::<Report>()
    );
    Ok(())
}
