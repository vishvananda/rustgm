//! Finite snapshots retain all attempts and distinguish independent sticky limits.
use super::*;
use crate::expand::config::Config;
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

#[test]
fn exact_finite_boundaries_keep_each_first_failure_and_failed_attempt() -> TestResult {
    let default = SourceContractLimits::default();
    assert_eq!(
        default,
        SourceContractLimits::new(1_000_000, 67_108_864, 67_108_864)
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
        let remaining = limits.work_limit() as usize - context.contracts().usage().0;
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
    assert_eq!(size_of::<SourceContractLimits>(), 12);
    assert_eq!(size_of::<Config>(), 80);
    assert_eq!(size_of::<State>(), 104);
    assert_eq!(size_of::<Contracts>(), 112);
    assert_eq!(size_of::<crate::resolve::bindings::SourceCore>(), 184);
    assert_eq!(size_of::<Package>(), 256);
    assert_eq!(size_of::<crate::syntax::ast::Node>(), 36);
    assert_eq!(size_of::<crate::body::ir::Body>(), 456);
}
