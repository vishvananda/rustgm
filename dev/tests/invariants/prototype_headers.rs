//! Unique declarations share recipes; actual signatures and execution remain independent.
use super::*;
use crate::support::{bindings::SourceIntrinsic, hash::HashSet};
type TestResult = Result<(), Box<dyn std::error::Error>>;
const PACKET: &str = "pa28/tests/spec/intrinsic-declarations";

// Source Reborrow storage belongs to the existing optional source catalog.
// Check ordinary records during metadata compilation as well as runtime. The
// once-per-crate header input bound adds 16 bytes, and the one pending Copy
// selection a declaration-time constant may await adds 8.
const _: [(); 7144] = [(); size_of::<Declarations>()];
const _: [(); 456] = [(); size_of::<crate::body::ir::Body>()];
const _: [(); 24] = [(); size_of::<crate::body::ir::Value>()];
const _: [(); 36] = [(); size_of::<crate::syntax::ast::Node>()];
const _: [(); 3768] = [(); size_of::<crate::sema::compiler_items::Catalog>()];

#[test]
fn registry_retains_distinct_identities_and_shared_finite_profiles() -> TestResult {
    let cases = crate::test_source::fixture(&format!("{PACKET}/cases.tsv"))?;
    let mut identities = HashSet::default();
    let mut profiles = HashSet::default();
    let directory = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../pa28/tests/spec/intrinsic-declarations/positive");
    for entry in std::fs::read_dir(directory)? {
        let entry = entry?;
        let name = entry.path();
        let name = name
            .file_stem()
            .and_then(|n| n.to_str())
            .ok_or("intrinsic fixture name")?;
        let Some(SourceIntrinsic::Declaration(identity)) = SourceIntrinsic::from_name(name) else {
            return Err(format!("missing unique intrinsic identity: {name}").into());
        };
        assert_eq!(identity.name(), name);
        assert!(identities.insert(identity));
        profiles.insert(std::ptr::from_ref(identity.profile()));
        assert!(identity.profile().work < 64);
    }
    assert_eq!((identities.len(), profiles.len()), (296, 100));
    assert_eq!(cases.lines().count(), 642);
    assert!(SourceIntrinsic::from_name("rustgm_unknown_intrinsic").is_none());
    assert_eq!(size_of::<SourceIntrinsic>(), 4);
    assert_eq!(size_of::<crate::syntax::ast::Node>(), 36);
    assert_eq!(size_of::<crate::body::ir::Body>(), 456);
    // The once-per-crate header input bound adds 16 bytes and the pending Copy
    // selection wait adds 8.
    assert_eq!(size_of::<Declarations>(), 7144);
    Ok(())
}

#[test]
fn actual_generic_primitive_headers_and_opponents_use_the_real_source_pipeline() -> TestResult {
    let cases = crate::test_source::fixture(&format!("{PACKET}/cases.tsv"))?;
    let mut positive = 0;
    let mut negative = 0;
    for line in cases.lines() {
        let (path, status) = line.split_once('\t').ok_or("intrinsic case")?;
        let source = crate::test_source::fixture(path)?;
        let result = crate::test_source::source_provider(&source, "intrinsic_headers")
            .and_then(|report| crate::pipeline::analyze_source(&report).map_err(|e| e.to_string()));
        if status == "0" {
            result.map_err(|e| format!("{path}: {e}"))?;
            positive += 1;
        } else {
            assert!(result.is_err(), "{path}");
            negative += 1;
        }
    }
    assert_eq!((positive, negative), (281, 361));
    Ok(())
}

#[test]
fn named_intrinsics_require_the_actual_nominal_role_in_the_completed_signature() -> TestResult {
    let cases = crate::test_source::fixture(&format!("{PACKET}/named-headers.tsv"))?;
    for line in cases.lines() {
        let (path, status) = line.split_once('\t').ok_or("nominal signature case")?;
        let source = crate::test_source::fixture(path)?;
        let report = crate::test_source::source_provider(&source, "intrinsic_nominal")?;
        let result = crate::pipeline::analyze_source(&report);
        if status == "0" {
            result.map_err(|e| format!("{path}: {e}"))?;
        } else {
            let error = result
                .err()
                .ok_or("intrinsic accepted a nominal lookalike")?
                .to_string();
            assert!(error.contains("nominal declaration"), "{path}: {error}");
        }
    }
    assert_eq!(cases.lines().count(), 17);
    Ok(())
}

#[test]
fn failed_structural_work_and_distinct_late_regions_cannot_publish_a_signature() -> TestResult {
    use crate::{
        support::bindings::prototype_specs::{Region, Term},
        types::{Kind, RegionId},
    };
    let source = crate::test_source::fixture(&format!("{PACKET}/foundation.rs"))?;
    let report = crate::test_source::source_provider(&source, "intrinsic_recipe")?;
    let facts = Declarations::new(&report)?;
    let mut exhausted = Matcher::new(&facts, [TypeId::UNIT; 4], 0, 0);
    assert!(exhausted.ty(Term::Unit, TypeId::UNIT).is_err());
    let mut facts = facts;
    let first = RegionId::bound(0, 0)?;
    let second = RegionId::bound(0, 1)?;
    let reference = |region| Kind::Reference {
        mutable: false,
        region,
        referent: TypeId::UNIT,
    };
    let a = facts.types.intern(reference(first))?;
    let b = facts.types.intern(reference(second))?;
    let mut matcher = Matcher::new(&facts, [TypeId::UNIT; 4], 2, 20);
    matcher.ty(Term::Reference(false, Region::Bound(2), &Term::Unit), a)?;
    matcher.ty(Term::Reference(false, Region::Bound(0), &Term::Unit), b)?;
    assert!(
        matcher
            .ty(Term::Reference(false, Region::Bound(2), &Term::Unit), b)
            .is_err()
    );
    let mut matcher = Matcher::new(&facts, [TypeId::UNIT; 4], 2, 20);
    matcher.ty(Term::Reference(false, Region::Bound(2), &Term::Unit), a)?;
    assert!(
        matcher
            .ty(Term::Reference(false, Region::Bound(0), &Term::Unit), a)
            .is_err()
    );
    Ok(())
}

fn executed_intrinsic(source: &str) -> Result<Result<(), String>, Box<dyn std::error::Error>> {
    let report = crate::test_source::source_provider(source, "intrinsic_execution")?;
    let result = crate::pipeline::analyze_source(&report).and_then(|analysis| {
        let crate::pipeline::Analysis {
            mut facts,
            prepared,
        } = analysis;
        crate::lower::instances::build_roots_prepared(
            &report,
            &mut facts,
            crate::lower::Roots {
                entry: None,
                exports: &[],
            },
            prepared,
        )
    });
    Ok(result.map(drop).map_err(|error| error.to_string()))
}

#[test]
fn reached_intrinsics_cannot_execute_their_authored_fallback() -> TestResult {
    // `ctpop` has a validated constant handler: `ctpop(7)` is 3, never the
    // authored fallback's 7. Each length is accepted only with the handler's.
    let constant = crate::test_source::fixture(&format!("{PACKET}/execution/const.rs"))?;
    for (length, accepted) in [(3, true), (7, false)] {
        let source = format!("{constant}fn length(a:[u8;ANSWER as usize])->[u8;{length}]{{a}}\n");
        match executed_intrinsic(&source)? {
            Ok(()) => assert!(accepted, "`ctpop` executed its fallback: [u8; {length}]"),
            Err(error) => assert!(
                !accepted && error.contains("incompatible types"),
                "[u8; {length}]: {error}"
            ),
        }
    }
    // An intrinsic with no constant handler keeps rejecting reached execution.
    let unhandled = constant
        .replace(
            "ctpop<T0:Copy>(a0:T0)->u32{7}",
            "bswap<T0:Copy>(a0:T0)->T0{a0}",
        )
        .replace("ctpop(7u32)", "bswap(7u32)");
    assert_ne!(
        unhandled, constant,
        "the unhandled intrinsic source changed"
    );
    for (name, source) in [
        ("unhandled const", unhandled),
        (
            "runtime",
            crate::test_source::fixture(&format!("{PACKET}/execution/runtime.rs"))?,
        ),
        (
            "reified",
            crate::test_source::fixture(&format!("{PACKET}/execution/reified.rs"))?,
        ),
    ] {
        let error = executed_intrinsic(&source)?
            .err()
            .ok_or("unsupported source intrinsic executed its fallback")?;
        assert!(
            error.contains("source intrinsic")
                && error.contains(if name == "reified" {
                    "cannot be coerced or cast"
                } else {
                    "not implemented"
                }),
            "{name}: {error}"
        );
    }
    Ok(())
}

#[test]
fn new_intrinsic_declarations_cannot_lose_their_contract_in_legacy_metadata() -> TestResult {
    let source = crate::test_source::fixture(&format!("{PACKET}/positive/ctpop.rs"))?;
    let report = crate::test_source::source_provider(&source, "intrinsic_metadata")?;
    let mut facts = crate::sema::declarations::analyze(&report)?;
    let identity = report
        .package
        .source_core()
        .ok_or("provider")?
        .current
        .clone();
    assert_eq!(
        crate::sema::crates::authored_export::Stage::begin(&report, &mut facts, &identity).err(),
        Some("source intrinsic declaration semantics require a wider metadata profile")
    );
    Ok(())
}
