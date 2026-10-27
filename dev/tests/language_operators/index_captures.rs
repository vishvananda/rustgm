//! Index captures consume existing receiver selections and preserve invocation effects.
use super::{mutation::frontend, native};
use rustgm::sema::closures::{AccessKind, captures::Mode};
use rustgm::types::closure::CallKind;
use std::path::Path;
type TestResult = Result<(), Box<dyn std::error::Error>>;
fn root() -> Result<&'static Path, Box<dyn std::error::Error>> {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or_else(|| "root".into())
}
fn source(name: &str) -> Result<String, Box<dyn std::error::Error>> {
    let path = if name == "captured-original" {
        "index-mut/captured".to_owned()
    } else {
        format!("index-capture/{name}")
    };
    Ok(std::fs::read_to_string(
        root()?.join(format!("pa26/tests/spec/{path}.rs")),
    )?)
}
const POSITIVE: &[&str] = &[
    "captured-original",
    "overloaded-shared",
    "overloaded-mutable",
    "ref-shared",
    "ref-deep",
    "ref-mutable",
    "move-ref-mutable",
    "nested-mutable",
    "dynamic-index-captured",
    "parameter-index",
    "index-order",
    "array-ref-shared",
    "array-ref-mutable",
    "slice-ref-shared",
    "slice-ref-mutable",
    "unique-ref-mutable",
    "shared-noncopy-output",
    "precise-field",
];
#[test]
fn actual_selected_index_capture_sources_reach_checked_lowir() -> TestResult {
    for &name in POSITIVE {
        frontend::checked(&source(name)?).map_err(|error| format!("{name}: {error}"))?;
    }
    for name in [
        "immutable-base",
        "shared-base",
        "live-loan",
        "immutable-closure",
        "missing-indexmut",
        "noncopy-output-move",
    ] {
        let result = frontend::checked(&source(name)?);
        let error = result
            .err()
            .ok_or_else(|| format!("{name}: invalid capture accepted"))?;
        println!("index-capture-negative={name} diagnostic={error}");
        assert!(!error.contains("not implemented"), "{name}: {error}");
    }
    Ok(())
}
#[test]
fn selected_receiver_modes_and_dereference_prefixes_remain_separate() -> TestResult {
    for (name, kind, mode, depth) in [
        ("overloaded-shared", CallKind::Shared, Mode::Shared, 0),
        ("overloaded-mutable", CallKind::Mutable, Mode::Mutable, 0),
        ("ref-shared", CallKind::Shared, Mode::Shared, 1),
        ("ref-deep", CallKind::Shared, Mode::Shared, 2),
        ("ref-mutable", CallKind::Mutable, Mode::Mutable, 1),
        ("move-ref-mutable", CallKind::Mutable, Mode::Value, 1),
        ("array-ref-shared", CallKind::Shared, Mode::Shared, 1),
        ("array-ref-mutable", CallKind::Mutable, Mode::Mutable, 1),
        ("slice-ref-shared", CallKind::Shared, Mode::Shared, 1),
        ("slice-ref-mutable", CallKind::Mutable, Mode::Mutable, 1),
        (
            "unique-ref-mutable",
            CallKind::Mutable,
            Mode::UniqueImmutable,
            1,
        ),
    ] {
        let (_, analysis) = frontend::analyze(&source(name)?)?;
        let body = analysis.facts.closures.all().first().ok_or("closure")?;
        let environment = analysis
            .facts
            .closures
            .environment(body.identity)
            .ok_or("environment")?;
        assert_eq!(
            analysis.facts.types.closure_kind(body.identity),
            Some(kind),
            "{name}"
        );
        assert_eq!(environment.plan.captures().len(), 1, "{name}");
        assert_eq!(environment.plan.captures()[0].mode, mode, "{name}");
        if depth == 0 {
            assert!(environment.adjusted_accesses.is_empty(), "{name}");
        } else {
            let accesses: Vec<_> = environment
                .adjusted_accesses
                .iter()
                .filter(|((_, kind), _)| *kind == AccessKind::Index)
                .collect();
            assert_eq!(accesses.len(), 1, "{name}");
            assert_eq!(accesses[0].1.dereferences, depth, "{name}");
            assert!(
                environment.plan.covering(accesses[0].1.path).is_some(),
                "{name}"
            );
        }
    }
    println!(
        "index-capture-layout access={} adjusted={} environment={} body={} operation={}",
        size_of::<AccessKind>(),
        size_of::<rustgm::sema::closures::AdjustedAccess>(),
        size_of::<rustgm::sema::closures::Environment>(),
        size_of::<rustgm::body::ir::Body>(),
        size_of::<rustgm::sema::model::Operation>()
    );
    assert_eq!(size_of::<AccessKind>(), 1);
    assert_eq!(size_of::<rustgm::sema::closures::AdjustedAccess>(), 8);
    assert_eq!(size_of::<rustgm::body::ir::Body>(), 456);
    assert_eq!(size_of::<rustgm::sema::model::Operation>(), 16);
    Ok(())
}
#[test]
fn repeated_index_sites_share_paths_with_bounded_work_and_sparse_storage() -> TestResult {
    for width in [1usize, 8, 32] {
        let (_, analysis) = frontend::analyze(&source(&format!("scaling-{width}"))?)?;
        let facts = &analysis.facts;
        let body = facts.closures.all().first().ok_or("scaling closure")?;
        let environment = facts
            .closures
            .environment(body.identity)
            .ok_or("scaling environment")?;
        assert_eq!(environment.plan.path_count(), 2);
        assert_eq!(environment.plan.captures().len(), 1);
        assert_eq!(environment.adjusted_accesses.len(), width);
        let work = facts.body_work.inference_steps
            + facts.body_work.canonical_steps
            + facts.body_work.steps;
        assert!(work < 50_000 + 5_000 * width, "{width}: {work}");
        println!(
            "index-capture width={width} inference-body-work={work} paths={} captures={} adjusted-len={} adjusted-capacity={} adjusted-capacity-payload-bytes={}",
            environment.plan.path_count(),
            environment.plan.captures().len(),
            environment.adjusted_accesses.len(),
            environment.adjusted_accesses.capacity(),
            environment.adjusted_accesses.capacity()
                * size_of::<(
                    (rustgm::resolve::unit::NodeKey, AccessKind),
                    rustgm::sema::closures::AdjustedAccess
                )>()
        );
    }
    let text = std::fs::read_to_string(
        root()?.join("pa24/tests/spec/typed-capture-uses/array-shared.rs"),
    )?;
    let (_, analysis) = frontend::analyze(&text)?;
    let body = analysis
        .facts
        .closures
        .all()
        .first()
        .ok_or("ordinary array closure")?;
    let environment = analysis
        .facts
        .closures
        .environment(body.identity)
        .ok_or("ordinary array environment")?;
    assert_eq!(environment.plan.path_count(), 1);
    assert!(environment.adjusted_accesses.is_empty());
    println!(
        "ordinary-array paths={} adjusted-len={} adjusted-capacity={}",
        environment.plan.path_count(),
        environment.adjusted_accesses.len(),
        environment.adjusted_accesses.capacity()
    );
    Ok(())
}
#[test]
#[ignore = "requires pinned Rust and the qualified CPPGM backend"]
fn captured_index_sources_execute_with_original_scalar_order_controls() -> TestResult {
    for &name in POSITIVE {
        let text = source(name)?;
        let body = text.strip_prefix(native::CHECK).ok_or("check preamble")?;
        native::check_cases(
            "parallel/pa26/selected-index-captures-v1/native",
            &[(name, body, false)],
        )?;
    }
    for name in ["store-order", "compound-order"] {
        let text =
            std::fs::read_to_string(root()?.join(format!("pa26/tests/spec/index-mut/{name}.rs")))?;
        native::check_cases(
            "parallel/pa26/selected-index-captures-v1/native",
            &[(
                name,
                text.strip_prefix(native::CHECK).ok_or("check preamble")?,
                false,
            )],
        )?;
    }
    let text =
        std::fs::read_to_string(root()?.join("pa24/tests/spec/typed-capture-native/indices.rs"))?;
    native::check_cases(
        "parallel/pa26/selected-index-captures-v1/native",
        &[("scalar-dynamic-indices", &text, false)],
    )?;
    Ok(())
}
