//! Finite snapshots retain all attempts and distinguish independent sticky limits.
//! The allowance grows only with admitted authenticated source; explicit values cap it.
use super::*;
use crate::expand::config::Config;
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

#[test]
fn exact_finite_boundaries_keep_each_first_failure_and_failed_attempt() -> TestResult {
    // Amended: defaults are no explicit caps; the admitted-input allowance governs.
    let default = SourceContractLimits::default();
    assert_eq!(
        (
            default.work_limit(),
            default.attempted_bytes(),
            default.retained_bytes()
        ),
        (None, None, None)
    );
    let partial = default.with_work_limit(1).with_retained_bytes(3);
    assert_eq!(
        (
            partial.work_limit(),
            partial.attempted_bytes(),
            partial.retained_bytes()
        ),
        (Some(1), None, Some(3))
    );
    assert_eq!(
        partial.with_attempted_bytes(2),
        SourceContractLimits::new(1, 2, 3)
    );
    for (limits, kind) in [
        (
            SourceContractLimits::new(1, u32::MAX, u32::MAX),
            Exhaustion::Work,
        ),
        (
            SourceContractLimits::new(u32::MAX, 1, u32::MAX),
            Exhaustion::Attempted,
        ),
        (
            SourceContractLimits::new(u32::MAX, u32::MAX, 1),
            Exhaustion::Retained,
        ),
    ] {
        let contracts = Contracts::new(limits);
        let result = match kind {
            Exhaustion::Work => {
                contracts.charge(1, 0)?;
                contracts.charge(1, 0)
            }
            Exhaustion::Attempted => {
                contracts.charge(0, 1)?;
                contracts.charge(0, 1)
            }
            Exhaustion::Retained => {
                contracts.memo_payload(1)?;
                contracts.memo_payload(2)
            }
        };
        assert_eq!(result, Err(kind.message()));
        let before = contracts.usage();
        assert_eq!(contracts.charge(7, 11), Err(kind.message()));
        assert_eq!(contracts.usage(), (before.0 + 7, before.1 + 11, before.2));
        assert_eq!(contracts.memo_payload(usize::MAX), Err(kind.message()));
        assert_eq!(contracts.charge(0, 0), Err(kind.message()));
    }
    for kind in [
        Exhaustion::Work,
        Exhaustion::Attempted,
        Exhaustion::Retained,
    ] {
        let contracts = Contracts::new(SourceContractLimits::new(u32::MAX, u32::MAX, u32::MAX));
        let maximum = u32::MAX as usize;
        let result = match kind {
            Exhaustion::Work => {
                contracts.charge(maximum, 0)?;
                contracts.charge(1, 0)
            }
            Exhaustion::Attempted => {
                contracts.charge(0, maximum)?;
                contracts.charge(0, 1)
            }
            Exhaustion::Retained => {
                contracts.memo_payload(maximum)?;
                contracts.memo_payload(maximum + 1)
            }
        };
        assert_eq!(result, Err(kind.message()), "maximum remains finite");
    }
    Ok(())
}

struct NoFiles;
impl crate::resolve::load::Sources for NoFiles {
    fn read(
        &mut self,
        _: &std::path::Path,
    ) -> Result<Option<crate::resolve::load::InputFile>, String> {
        Ok(None)
    }
}
fn report(
    source: &str,
    provider: bool,
    config: &Config,
) -> TestResult<crate::resolve::expansion::Report> {
    use crate::{
        resolve,
        support::{bindings::Mode, crates::CrateIdentity, source::SourceFile},
    };
    let identity = CrateIdentity::new("minicore", "contract_limits")?;
    let context = if provider {
        resolve::bindings::Context {
            crate_name: Some("minicore".into()),
            current: Some(identity.clone()),
            mode: Mode::Source { provider: identity },
            ..Default::default()
        }
    } else {
        resolve::bindings::Context::default()
    };
    Ok(resolve::expansion::analyze_with_context(
        resolve::load::InputFile {
            identity: "lib.rs".into(),
            path: "lib.rs".into(),
            source: SourceFile::new(source.as_bytes().to_vec())?,
        },
        &mut NoFiles,
        config,
        &context,
    )?)
}
#[test]
fn both_actual_owner_constructors_keep_snapshot_and_warm_denial() -> TestResult {
    for (name, provider) in [("ordinary", false), ("source", true)] {
        let source = crate::test_source::fixture(&format!(
            "pa31/tests/spec/source-contract-limits/{name}.rs"
        ))?;
        let limits = SourceContractLimits::new(123_456, 2_000_000, 1_000_000);
        let mut config = Config::default();
        config.set_source_contract_limits(limits);
        let report = report(&source, provider, &config)?;
        let context = report
            .package
            .macro_context()?
            .ok_or("actual contract owner")?;
        assert_eq!(context.contracts().0.borrow().limits, limits);
        assert_eq!(report.package.source_core().is_some(), provider);
        config.set_source_contract_limits(SourceContractLimits::new(0, 0, 0));
        assert_eq!(context.contracts().0.borrow().limits, limits);
        let key = context
            .contracts()
            .0
            .borrow()
            .declarations
            .keys()
            .next()
            .copied()
            .ok_or("actual declaration")?;
        assert!(checked_definition(&report.package, key)?.is_some());
        let cap = limits.work_limit().ok_or("explicit work cap")? as usize;
        assert_eq!(
            context.contracts().0.borrow().granted.capped(limits).work,
            cap
        );
        let remaining = cap - context.contracts().usage().0;
        assert_eq!(
            context.contracts().charge(remaining + 1, 0),
            Err(Exhaustion::Work.message())
        );
        assert!(checked_definition(&report.package, key).is_err());
        for (limits, kind) in [
            (
                SourceContractLimits::new(0, u32::MAX, u32::MAX),
                Exhaustion::Work,
            ),
            (
                SourceContractLimits::new(u32::MAX, 0, u32::MAX),
                Exhaustion::Attempted,
            ),
            (
                SourceContractLimits::new(u32::MAX, u32::MAX, 0),
                Exhaustion::Retained,
            ),
        ] {
            config.set_source_contract_limits(limits);
            let result = self::report(&source, provider, &config);
            assert!(
                result
                    .err()
                    .ok_or("real zero budget accepted")?
                    .to_string()
                    .contains(kind.message()),
                "{name}"
            );
        }
    }
    let mut config = Config::default();
    config.set_source_contract_limits(SourceContractLimits::new(0, 0, 0));
    let ordinary = report("pub fn value()->u8{7u8}", false, &config)?;
    assert!(ordinary.package.macro_context()?.is_none());
    Ok(())
}
#[test]
fn immutable_limit_layout_is_cold_and_default_records_remain_dense() {
    eprintln!(
        "limits={} Config={} State={} Contracts={} SourceCore={} Source={} Package={} Node={} Body={}",
        size_of::<SourceContractLimits>(),
        size_of::<Config>(),
        size_of::<State>(),
        size_of::<Contracts>(),
        size_of::<crate::resolve::bindings::SourceCore>(),
        size_of::<crate::resolve::bindings::Source>(),
        size_of::<Package>(),
        size_of::<crate::syntax::ast::Node>(),
        size_of::<crate::body::ir::Body>()
    );
    // Amended: optional caps plus the granted allowance live once per ledger owner.
    assert_eq!(size_of::<SourceContractLimits>(), 24);
    assert_eq!(size_of::<Config>(), 96);
    assert_eq!(size_of::<State>(), 152);
    assert_eq!(size_of::<Contracts>(), 160);
    assert_eq!(size_of::<crate::resolve::bindings::SourceCore>(), 240);
    assert_eq!(size_of::<Package>(), 256);
    assert_eq!(size_of::<crate::syntax::ast::Node>(), 36);
    assert_eq!(size_of::<crate::body::ir::Body>(), 456);
}

const CORE: &str = "#![feature(no_core,lang_items,rustc_attrs)]#![no_core]#[lang=\"pointee_sized\"]trait PointeeSized{}#[lang=\"meta_sized\"]trait MetaSized:PointeeSized{}#[lang=\"sized\"]trait Sized:MetaSized{}#[rustc_builtin_macro]macro_rules!include{($($t:tt)*)=>{}}";
const PIECE: &str = "#[allow(unused)]const P:u8=1;";
const UNIT: &str = "#[allow(unused)]const U:u8=2;";
/// Aliases resolve to one canonical identity, as the driver supplies them.
struct Aliased;
impl crate::resolve::load::Sources for Aliased {
    fn read(
        &mut self,
        path: &std::path::Path,
    ) -> Result<Option<crate::resolve::load::InputFile>, String> {
        let canonical = path.strip_prefix("./").unwrap_or(path);
        let text = match canonical.to_str() {
            Some("piece.rs") => PIECE,
            Some("unit.rs") => UNIT,
            _ => return Ok(None),
        };
        Ok(Some(crate::resolve::load::InputFile {
            identity: canonical.into(),
            path: path.into(),
            source: crate::support::source::SourceFile::new(text.as_bytes().to_vec())?,
        }))
    }
}
fn provider(source: &str, config: &Config) -> TestResult<crate::resolve::expansion::Report> {
    use crate::support::{bindings::Mode, crates::CrateIdentity, source::SourceFile};
    let identity = CrateIdentity::new("minicore", "contract_grants")?;
    Ok(crate::resolve::expansion::analyze_with_context(
        crate::resolve::load::InputFile {
            identity: "lib.rs".into(),
            path: "lib.rs".into(),
            source: SourceFile::new(source.as_bytes().to_vec())?,
        },
        &mut Aliased,
        config,
        &crate::resolve::bindings::Context {
            crate_name: Some("minicore".into()),
            current: Some(identity.clone()),
            mode: Mode::Source { provider: identity },
            ..Default::default()
        },
    )?)
}
fn granted(report: &crate::resolve::expansion::Report) -> TestResult<allowance::Allowance> {
    let context = report.package.macro_context()?.ok_or("contract owner")?;
    Ok(context.contracts().0.borrow().granted)
}
#[test]
fn each_admitted_identity_grants_once_and_expansion_output_grants_nothing() -> TestResult {
    for copies in [1, 4, 32] {
        let mut source = CORE.to_owned();
        for copy in 0..copies {
            use std::fmt::Write as _;
            let alias = if copy % 2 == 0 { "" } else { "./" };
            write!(
                source,
                "mod i{copy}{{include!(\"{alias}piece.rs\");}}#[path=\"{alias}unit.rs\"]mod u{copy};"
            )?;
        }
        source.push_str(
            "macro_rules!more{()=>{mod generated{include!(\"piece.rs\");}#[path=\"unit.rs\"]mod generated_unit;};}more!{}",
        );
        let report = provider(&source, &Config::default())?;
        let actual = granted(&report)?;
        let admitted = source.len() + PIECE.len() + UNIT.len();
        // Copies of one identity, aliases and macro-generated requests add nothing.
        assert_eq!(
            actual,
            allowance::Allowance::SETUP.admit(admitted),
            "{copies}"
        );
        assert!(report.package.units.len() > copies + 2, "{copies}");
    }
    Ok(())
}
#[test]
fn explicit_caps_bound_the_grant_and_grants_never_reset_failure() -> TestResult {
    let source = crate::support::source::SourceFile::new(vec![b' '; 1000])?;
    let grant = allowance::Allowance::SETUP.admit(1000);
    for (limits, work) in [
        (SourceContractLimits::default(), grant.work),
        (SourceContractLimits::default().with_work_limit(5), 5),
        (
            SourceContractLimits::default().with_work_limit(u32::MAX),
            grant.work,
        ),
    ] {
        let contracts = Contracts::admitted(limits, &source);
        assert_eq!(contracts.0.borrow().granted.capped(limits).work, work);
        contracts.charge(work, 0)?;
        assert_eq!(contracts.charge(1, 0), Err(Exhaustion::Work.message()));
        // A later admission neither refunds the failed attempt nor clears it.
        contracts.admit(&source)?;
        assert!(contracts.0.borrow().granted.work > grant.work);
        assert_eq!(contracts.usage().0, work + 1);
        assert_eq!(contracts.charge(0, 0), Err(Exhaustion::Work.message()));
    }
    let capped = SourceContractLimits::default()
        .with_attempted_bytes(3)
        .with_retained_bytes(2);
    let contracts = Contracts::admitted(capped, &source);
    assert_eq!(contracts.charge(0, 4), Err(Exhaustion::Attempted.message()));
    let contracts = Contracts::admitted(capped, &source);
    assert_eq!(
        contracts.memo_payload(3),
        Err(Exhaustion::Retained.message())
    );
    Ok(())
}
/// One source line per macro invocation; work must stay proportional to bytes.
fn invocations(count: usize) -> String {
    let mut source = format!("{CORE}macro_rules!m{{()=>{{}};}}");
    for _ in 0..count {
        source.push_str("m!{}\n");
    }
    source
}
#[test]
fn linear_invocation_family_scales_and_exponential_expansion_still_fails() -> TestResult {
    let mut ratios = Vec::new();
    for count in [512, 4096] {
        let source = invocations(count);
        let report = provider(&source, &Config::default())?;
        let (work, attempted, _) = report
            .package
            .macro_context()?
            .ok_or("contract owner")?
            .contracts()
            .usage();
        let bytes = source.len();
        eprintln!("invocations={count} bytes={bytes} work={work} attempted={attempted}");
        assert_eq!(granted(&report)?, allowance::Allowance::SETUP.admit(bytes));
        ratios.push(work / count);
        assert!(work <= 2 * bytes, "{count}: {work} work for {bytes} bytes");
    }
    assert!(4 * ratios[1] <= 5 * ratios[0], "{ratios:?}");
    let doubling = |depth: usize| {
        format!(
            "{CORE}macro_rules!e{{()=>{{}};(x $($t:tt)*)=>{{e!{{$($t)*}}e!{{$($t)*}}}};}}e!{{{}}}",
            "x ".repeat(depth)
        )
    };
    // Exponential output grants nothing: the same admitted bytes pay 2^depth calls.
    let mut usage = Vec::new();
    for depth in [9, 12] {
        let source = doubling(depth);
        let report = provider(&source, &Config::default())?;
        assert_eq!(
            granted(&report)?,
            allowance::Allowance::SETUP.admit(source.len())
        );
        let context = report.package.macro_context()?.ok_or("contract owner")?;
        usage.push(context.contracts().usage().0);
    }
    eprintln!("doubling work at depths 9 and 12: {usage:?}");
    assert!(usage[1] >= 7 * usage[0], "{usage:?}");
    let error = provider(&doubling(40), &Config::default())
        .err()
        .ok_or("exponential expansion accepted")?
        .to_string();
    assert!(
        error.contains("budget exceeded") || error.contains("exceed their"),
        "{error}"
    );
    Ok(())
}
