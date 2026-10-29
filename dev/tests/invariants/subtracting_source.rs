//! Genuine source arithmetic keeps its declaration, caller, const and Copy proofs.
use super::*;
use crate::{
    pipeline,
    resolve::body_origin::BodyOrigin,
    sema::{
        header_attributes,
        values::{Reader, View},
    },
    types::instance::Instance,
};
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
const TYPES: [&str; 12] = [
    "u8", "u16", "u32", "u64", "u128", "usize", "i8", "i16", "i32", "i64", "i128", "isize",
];
fn source(name: &str) -> TestResult<String> {
    Ok(crate::test_source::fixture(&format!(
        "pa28/tests/spec/source-subtraction/{name}.rs"
    ))?)
}
fn report(name: &str) -> TestResult<Report> {
    Ok(crate::test_source::source_provider(
        &source(name)?,
        "source_subtraction",
    )?)
}
fn named(report: &Report, name: &str) -> TestResult<DefId> {
    report
        .resolution
        .definitions
        .iter()
        .enumerate()
        .find_map(|(index, d)| {
            (report.package.names.get(d.name) == name)
                .then(|| u32::try_from(index).ok().map(DefId))
                .flatten()
        })
        .ok_or_else(|| format!("missing actual {name}").into())
}
fn produce(name: &str) -> TestResult {
    let report = report(name)?;
    let pipeline::Analysis {
        mut facts,
        prepared,
    } = pipeline::analyze_source(&report)?;
    let program = crate::lower::instances::build_roots_prepared(
        &report,
        &mut facts,
        crate::lower::Roots {
            entry: None,
            exports: &[],
        },
        prepared,
    )?;
    for ty in TYPES {
        for prefix in ["unchecked", "wrapping", "overflow", "folded", "folded_flag"] {
            let symbol = format!("{prefix}_{ty}");
            assert_eq!(
                program
                    .symbols
                    .iter()
                    .filter(|s| s.name == symbol && s.body.is_some())
                    .count(),
                1,
                "{name}: {symbol}"
            );
        }
    }
    assert_eq!(
        program
            .symbols
            .iter()
            .filter(|s| s.name == "cold_observed" && s.body.is_some())
            .count(),
        1
    );
    let folder = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../obj/parallel/source-subtraction-v1/candidate")
        .join(name);
    std::fs::create_dir_all(&folder)?;
    std::fs::write(folder.join("source.rs"), source(name)?)?;
    crate::lowir::writer::write(
        &program,
        &mut std::fs::File::create(folder.join("provider.lowir"))?,
    )?;
    Ok(())
}
#[test]
fn genuine_source_subtraction_all_integer_widths_and_cold_body_produce_native_inputs() -> TestResult
{
    produce("source-runtime")
}
#[test]
fn generic_copy_forwarding_keeps_symbolic_formation_until_real_integer_instance() -> TestResult {
    // Pinned Rust metadata accepts this source; its object-stage ICE is retained separately.
    produce("generic-forward")
}
#[test]
fn unmarked_same_name_functions_execute_their_actual_ordinary_source_bodies() -> TestResult {
    produce("ordinary-lookalike")
}
#[test]
fn actual_source_constants_return_wrapping_bits_and_overflow_with_checked_faults() -> TestResult {
    let report = report("source-runtime")?;
    let analysis = pipeline::analyze_source(&report)?;
    let reader = Reader::new(&analysis.facts);
    for ty in TYPES {
        let value = reader
            .definition(named(&report, &format!("VALUE_{ty}"))?)
            .ok_or("actual constant result")?;
        assert!(
            matches!(
                reader.view(value)?,
                View::Scalar(crate::sema::model::Scalar::Integer(5))
            ),
            "{ty}"
        );
        let pair = reader
            .definition(named(&report, &format!("PAIR_{ty}"))?)
            .ok_or("actual constant pair")?;
        let bits = if ty.ends_with("size") {
            64
        } else {
            ty[1..].parse::<u16>()?
        };
        let maximum = if ty.starts_with('i') {
            (1u128 << (bits - 1)) - 1
        } else {
            u128::MAX >> (128 - bits)
        };
        assert!(
            matches!(reader.view(reader.field(pair,0)?)?,View::Scalar(crate::sema::model::Scalar::Integer(n)) if n==maximum),
            "{ty}"
        );
        assert!(
            matches!(
                reader.view(reader.field(pair, 1)?)?,
                View::Scalar(crate::sema::model::Scalar::Boolean(true))
            ),
            "{ty}"
        );
    }
    Ok(())
}
#[test]
fn unsafe_const_copy_signature_and_scalar_opponents_cannot_gain_intrinsic_execution() -> TestResult
{
    for name in [
        "unsafe-call",
        "const-unsigned-overflow",
        "const-signed-overflow",
        "nonconst",
        "wrong-signature",
        "missing-copy",
        "wrong-safety",
        "wrong-result",
        "wrong-cold-body",
        "noncopy-argument",
        "invalid-type",
    ] {
        let result = report(name).and_then(|report| {
            let pipeline::Analysis {
                mut facts,
                prepared,
            } = pipeline::analyze_source(&report)?;
            let exports = report
                .resolution
                .definitions
                .iter()
                .enumerate()
                .filter_map(|(i, d)| {
                    (d.node.is_some() && report.package.names.get(d.name) == "wrong")
                        .then(|| u32::try_from(i).ok().map(DefId))
                        .flatten()
                })
                .collect::<Vec<_>>();
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
            .map_err(Into::into)
        });
        let error = result
            .err()
            .ok_or_else(|| format!("unsupported {name} executed"))?
            .to_string();
        assert!(
            !error.contains("unsupported source intrinsic"),
            "{name}: {error}"
        );
        assert!(!error.contains("budget"), "{name}: {error}");
        eprintln!("source subtraction {name}: {error}");
    }
    let missing =
        source("source-runtime")?.replace("intrinsic_fallback_is_spec", "ordinary_tool_metadata");
    let report = crate::test_source::source_provider(&missing, "source_subtraction")?;
    assert!(
        pipeline::analyze_source(&report).is_err(),
        "cold hint invented fallback authority"
    );
    Ok(())
}
fn actual_instance(facts: &Declarations) -> TestResult<(Instance, BodyOrigin)> {
    let origin = BodyOrigin::Source(
        facts
            .functions
            .iter()
            .find_map(|f| f.body)
            .ok_or("actual source body")?,
    );
    for i in 0..facts.types.len() {
        let ty = TypeId(u32::try_from(i)?);
        let Some(Kind::FunctionItem {
            item, arguments, ..
        }) = facts.types.kind(ty)
        else {
            continue;
        };
        if facts
            .source_bindings
            .as_deref()
            .ok_or("catalog")?
            .intrinsic_kind(DefId(item.0))?
            != Some(SourceIntrinsic::UncheckedSub)
        {
            continue;
        }
        if facts.types.arguments(arguments).is_some_and(|mut a| {
            a.next() == Some(Argument::Type(TypeId::primitive(Primitive::U8))) && a.next().is_none()
        }) {
            return Ok((Instance::new(&facts.types, ty)?, origin));
        }
    }
    Err("actual closed subtraction instance".into())
}
#[test]
fn current_header_instance_work_payments_and_ordinary_widths_remain_bounded() -> TestResult {
    let text = source("source-runtime")?;
    let report = crate::test_source::source_provider(&text, "source_subtraction")?;
    let pipeline::Analysis { mut facts, .. } = pipeline::analyze_source(&report)?;
    let (instance, origin) = actual_instance(&facts)?;
    let catalog = facts.source_bindings.as_deref().ok_or("source catalog")?;
    let mut selected = catalog
        .subtraction_selection(&facts, instance, origin)?
        .ok_or("actual selected instance")?;
    catalog.require_subtraction_report(&report, &facts, &selected)?;
    let foreign = crate::test_source::source_provider(&text, "source_subtraction")?;
    assert!(
        catalog
            .require_subtraction_report(&foreign, &facts, &selected)
            .is_err()
    );
    let output = std::mem::replace(&mut selected.output, TypeId::UNIT);
    assert!(
        catalog
            .require_subtraction_report(&report, &facts, &selected)
            .is_err()
    );
    selected.output = output;
    catalog.require_subtraction_report(&report, &facts, &selected)?;
    let mut previous = None;
    for count in [8, 32, 128] {
        let before = header_attributes::source_contract_storage(&facts).ok_or("header payer")?;
        for _ in 0..count {
            let selected = catalog
                .subtraction_selection(&facts, instance, origin)?
                .ok_or("selected")?;
            catalog.require_subtraction_report(&report, &facts, &selected)?;
        }
        let after = header_attributes::source_contract_storage(&facts).ok_or("header payer")?;
        assert_eq!(
            (before.1, before.2, before.3, before.4),
            (after.1, after.2, after.3, after.4)
        );
        let work = after.0 - before.0;
        if let Some(old) = previous {
            assert_eq!(work, old * 4);
        }
        previous = Some(work);
        eprintln!("source subtraction count={count} work={work} storage={after:?}");
    }
    let identity = report
        .package
        .source_core()
        .ok_or("actual source")?
        .provider
        .clone();
    assert!(
        crate::sema::crates::authored_export::Stage::begin_module_feature_predicates(
            &report, &mut facts, &identity
        )
        .is_err(),
        "new intrinsic metadata escaped its profile"
    );
    assert!(
        header_attributes::charge_source_contract_query(
            &facts,
            crate::support::crates::MAX_COMPONENTS
        )
        .is_err()
    );
    let before = header_attributes::source_contract_storage(&facts).ok_or("failed payer")?;
    assert!(
        facts
            .source_bindings
            .as_deref()
            .ok_or("catalog")?
            .subtraction_selection(&facts, instance, origin)
            .is_err()
    );
    let after = header_attributes::source_contract_storage(&facts).ok_or("failed payer")?;
    assert!(after.0 > before.0);
    assert_eq!(after.1, before.1);
    assert_eq!(size_of::<crate::body::ir::Body>(), 456);
    assert_eq!(size_of::<crate::syntax::ast::Node>(), 36);
    assert_eq!(size_of::<crate::sema::model::NodeFacts>(), 80);
    eprintln!(
        "subtraction layouts selection={} Catalog={} SourceIntrinsic={} Body={} Node={} NodeFacts={}",
        size_of::<SubtractionSelection>(),
        size_of::<Catalog>(),
        size_of::<SourceIntrinsic>(),
        size_of::<crate::body::ir::Body>(),
        size_of::<crate::syntax::ast::Node>(),
        size_of::<crate::sema::model::NodeFacts>()
    );
    Ok(())
}
