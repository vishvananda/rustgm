//! Source and API inventories pay one configured owner without sharing permission.
use super::*;
use crate::{
    expand::config::{Config, SourceContractLimits},
    resolve::{
        self,
        bindings::{Context, panic_contracts::Contracts},
        load::{InputFile, Sources},
    },
    support::{bindings::Mode, crates::CrateIdentity, diagnostics::LintCap, source::SourceFile},
};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

#[test]
fn inventory_and_prior_source_work_share_one_finite_sticky_payer() -> Result {
    for (limit, success) in [(1_000_000, false), (1_500_000, true)] {
        let payer = Contracts::new(SourceContractLimits::new(limit, 100, 100));
        payer.charge(700_000, 3)?;
        let mut draft = Draft {
            payer: Some(&payer),
            ..Draft::default()
        };
        assert_eq!(draft.charge(400_000, 5).is_ok(), success);
        assert_eq!(payer.usage(), (1_100_000, 8, 0));
        assert_eq!(draft.steps, 400_000);
        if !success {
            let error = draft.charge(7, 11).err().ok_or("sticky work failure")?;
            assert!(error.to_string().contains("work budget"));
            assert_eq!(payer.usage(), (1_100_007, 19, 0));
            assert!(draft.charge(0, 0).is_err());
        }
    }
    let payer = Contracts::new(SourceContractLimits::new(100, 7, 100));
    payer.charge(0, 3)?;
    let mut draft = Draft {
        payer: Some(&payer),
        ..Draft::default()
    };
    let error = draft
        .charge(1, 5)
        .err()
        .ok_or("attempted storage failure")?;
    assert!(error.to_string().contains("attempted storage budget"));
    assert_eq!(payer.usage(), (1, 8, 0));
    assert!(draft.charge(0, 0).is_err());
    Ok(())
}

#[test]
fn inventory_retention_persists_across_later_memo_queries_and_completion_release() -> Result {
    let payer = Contracts::new(SourceContractLimits::new(100, 100, 10));
    payer.inventory_charge(1, 8, 8)?;
    payer.memo_payload(2)?;
    // Completion can release capture storage, but its remaining rows stay paid.
    payer.inventory_charge(0, 0, 5)?;
    payer.memo_payload(5)?;
    assert_eq!(
        payer.memo_payload(6),
        Err("source macro contracts exceed their retained storage budget")
    );
    assert!(payer.inventory_charge(7, 11, 0).is_err());
    assert_eq!(payer.usage(), (8, 19, 0));
    assert!(payer.charge(0, 0).is_err());
    let payer = Contracts::new(SourceContractLimits::new(100, 100, 4));
    let mut draft = Draft {
        payer: Some(&payer),
        ..Draft::default()
    };
    assert!(
        draft
            .charge(1, 5)
            .err()
            .ok_or("retained failure")?
            .to_string()
            .contains("retained storage budget")
    );
    assert!(draft.charge(0, 0).is_err());
    Ok(())
}

struct NoFiles;
impl Sources for NoFiles {
    fn read(&mut self, _: &std::path::Path) -> std::result::Result<Option<InputFile>, String> {
        Ok(None)
    }
}

#[test]
fn actual_source_and_local_owners_retain_completed_policy_cost_without_new_permission() -> Result {
    for (name, provider, owner) in [
        ("provider", true, true),
        ("local", false, true),
        ("standalone", false, false),
    ] {
        let source =
            crate::test_source::fixture(&format!("pa31/tests/spec/inventory-budget/{name}.rs"))?;
        let mut config = Config::default();
        config.set_lint_cap(LintCap::Allow);
        config.set_source_contract_limits(SourceContractLimits::new(
            14_500_624,
            1_073_741_824,
            67_108_864,
        ));
        let identity = CrateIdentity::new("minicore", "inventory_budget")?;
        let context = if provider {
            Context {
                crate_name: Some("minicore".into()),
                current: Some(identity.clone()),
                mode: Mode::Source { provider: identity },
                ..Context::default()
            }
        } else {
            Context::default()
        };
        let expanded = resolve::expansion::analyze_with_context(
            InputFile {
                identity: "lib.rs".into(),
                path: "lib.rs".into(),
                source: SourceFile::new(source.into_bytes())?,
            },
            &mut NoFiles,
            &config,
            &context,
        )?;
        let before = expanded
            .package
            .macro_context()?
            .map(|c| c.contracts().usage());
        let report = resolve::analyze_expanded(expanded)?;
        let library = report.package.library_policy().ok_or("actual library")?;
        let inventory = library.inventory().ok_or("completed inventory")?;
        assert!(!inventory.policies().is_empty());
        // A repeated completion must not replace an existing live reservation.
        assert!(Draft::for_package(&report.package).is_err());
        assert_eq!(report.package.macro_context()?.is_some(), owner);
        if let Some(context) = report.package.macro_context()? {
            let payer = context.contracts();
            assert!(payer.usage().0 >= before.ok_or("prior source work")?.0 + inventory.steps());
            let mut retained = Retained::default();
            library.charge_retained(&mut retained)?;
            // No per-API permission is inferred from paying for that inventory.
            assert!(!report.package.requested_negative_impls()?);
            assert!(
                payer
                    .memo_payload(
                        67_108_864usize
                            .saturating_sub(retained.bytes)
                            .saturating_add(1)
                    )
                    .is_err()
            );
        }
    }
    assert_eq!(size_of::<Package>(), 256);
    assert_eq!(size_of::<crate::body::ir::Body>(), 456);
    Ok(())
}
