//! Actual source work, attempted scratch and transactional label publication.
use super::*;
use crate::expand::config::SourceContractLimits;
type TestResult = Result<(), Box<dyn std::error::Error>>;
fn inventory(steps: usize) -> Inventory {
    Inventory {
        policies: Vec::new(),
        relationships: Vec::new(),
        declarations: Vec::new(),
        imports: Vec::new(),
        consts: None,
        fields: None,
        steps,
    }
}
fn library() -> Result<Library, &'static str> {
    let mut library = Library::default();
    for name in ["zeta", "alpha_long", "alpha", "middle"] {
        library.request(name)?;
    }
    Ok(library)
}
fn labels(library: &Library) -> Vec<&str> {
    library.requests.iter().map(AsRef::as_ref).collect()
}
#[test]
fn source_ordering_keeps_the_existing_paid_history_above_the_metadata_ceiling() -> TestResult {
    let payer = Contracts::new(SourceContractLimits::new(1_500_000, 4096, 4096));
    payer.inventory_charge(1_100_000, 0, 128)?;
    let mut inventory = inventory(1_100_000);
    let mut library = library()?;
    library.prepare_request_order(&mut inventory, Some(&payer))?;
    assert_eq!(labels(&library), ["alpha", "alpha_long", "middle", "zeta"]);
    assert_eq!(payer.usage().0, inventory.steps);
    assert!(inventory.steps > 1_100_000 && inventory.steps < 1_101_000);
    assert_eq!(
        payer.usage().1,
        4 * 3 * size_of::<usize>() + 3 * size_of::<Vec<usize>>()
    );
    library.publish(inventory);
    let mut lookup = 0;
    for (name, expected) in [
        ("alpha", true),
        ("alpha_long", true),
        ("absent", false),
        ("zeta", true),
    ] {
        assert_eq!(
            library.requested_feature(
                &crate::resolve::bindings::FeatureName::new(name)?,
                &mut lookup
            )?,
            expected
        );
    }
    Ok(())
}
#[test]
fn failed_ordering_leaves_labels_unpublished_and_payment_sticky() -> TestResult {
    let complete = Contracts::new(SourceContractLimits::new(1_500_000, 4096, 4096));
    let mut ordered = library()?;
    ordered.prepare_request_order(&mut inventory(0), Some(&complete))?;
    // Interrupt every paid step, including comparisons after index permutation.
    for remaining in 0..complete.usage().0 {
        let mut library = library()?;
        let before = ["zeta", "alpha_long", "alpha", "middle"];
        let limit = 1_100_000 + u32::try_from(remaining)?;
        let payer = Contracts::new(SourceContractLimits::new(limit, 4096, 4096));
        payer.inventory_charge(1_100_000, 0, 0)?;
        let mut inventory = inventory(1_100_000);
        assert!(
            library
                .prepare_request_order(&mut inventory, Some(&payer))
                .is_err()
        );
        assert_eq!(labels(&library), before);
        assert!(payer.usage().0 > limit as usize);
        let attempted = payer.usage();
        assert!(
            library
                .prepare_request_order(&mut inventory, Some(&payer))
                .is_err()
        );
        assert!(payer.usage().0 > attempted.0);
        assert_eq!(labels(&library), before);
        assert!(payer.charge(0, 0).is_err());
        assert!(library.inventory.is_none());
    }
    Ok(())
}
#[test]
fn scratch_preflight_pays_attempts_and_never_releases_the_retained_inventory() -> TestResult {
    for (attempted_limit, retained_limit) in [(100, 1000), (1000, 200)] {
        let payer = Contracts::new(SourceContractLimits::new(
            1000,
            attempted_limit,
            retained_limit,
        ));
        payer.inventory_charge(0, 0, 64)?;
        let mut library = library()?;
        let mut inventory = inventory(0);
        assert!(
            library
                .prepare_request_order(&mut inventory, Some(&payer))
                .is_err()
        );
        assert_eq!(
            payer.usage().1,
            4 * 3 * size_of::<usize>() + 3 * size_of::<Vec<usize>>()
        );
        assert_eq!(labels(&library), ["zeta", "alpha_long", "alpha", "middle"]);
        assert!(payer.charge(0, 0).is_err());
    }
    let payer = Contracts::new(SourceContractLimits::new(1000, 1000, 256));
    payer.inventory_charge(0, 0, 64)?;
    let mut library = library()?;
    library.prepare_request_order(&mut inventory(0), Some(&payer))?;
    // Ordering did not erase the retained64 or persist its own transient168.
    assert!(payer.inventory_order_charge(0, 0, 192).is_ok());
    assert!(payer.inventory_order_charge(0, 0, 193).is_err());
    Ok(())
}
#[test]
fn ordinary_ordering_and_lookup_keep_their_own_default_cumulative_limit() -> TestResult {
    let mut library = library()?;
    assert!(
        library
            .prepare_request_order(&mut inventory(MAX_COMPONENTS), None)
            .is_err()
    );
    assert_eq!(labels(&library), ["zeta", "alpha_long", "alpha", "middle"]);
    library.publish(inventory(0));
    let feature = crate::resolve::bindings::FeatureName::new("alpha")?;
    let mut work = MAX_COMPONENTS;
    assert!(library.requested_feature(&feature, &mut work).is_err());
    assert!(work > MAX_COMPONENTS);
    assert!(library.requested_feature(&feature, &mut work).is_err());
    Ok(())
}
