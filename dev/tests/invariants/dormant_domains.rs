//! Actual reserved cells stay private to their moved report, with all opponents real.
use super::*;
use crate::{
    support::bindings::LangItem,
    unpaid_cases::{Result, actual},
};

#[test]
fn unpaid_reservations_move_full_family_without_names_roles_or_import_publication() -> Result {
    let (report, _facts, inventory) = actual(true)?;
    let before = report.resolution.definitions.len();
    let ast = report.package.units[0].configured.ast.nodes.as_ptr();
    let bindings = report
        .resolution
        .bindings
        .iter()
        .map(|(scope, name, namespace, binding)| (scope, name, namespace, *binding))
        .collect::<Vec<_>>();
    let roles = LangItem::ALL.map(|role| {
        report
            .resolution
            .source_bindings
            .as_deref()
            .and_then(|bindings| bindings.language(role))
            .map(|row| row.definition)
    });
    assert!(report.resolution.crates.is_none());
    let mut work = ReservationWork::default();
    let inputs = Inputs::reserve(report, Arc::clone(&inventory), 0, &mut work)?;
    assert_eq!(inputs.catalog.body_count(), 3);
    assert_eq!(inputs.definitions.start, before);
    assert_eq!(
        inputs.report.package.units[0].configured.ast.nodes.as_ptr(),
        ast
    );
    assert!(inputs.report.resolution.crates.is_none());
    assert_eq!(
        inputs
            .report
            .resolution
            .bindings
            .iter()
            .map(|(scope, name, namespace, binding)| (scope, name, namespace, *binding))
            .collect::<Vec<_>>(),
        bindings
    );
    assert_eq!(
        LangItem::ALL.map(|role| inputs
            .report
            .resolution
            .source_bindings
            .as_deref()
            .and_then(|bindings| bindings.language(role))
            .map(|row| row.definition)),
        roles
    );
    assert!(
        inputs.report.resolution.definitions[inputs.definitions.clone()]
            .iter()
            .all(|row| row.node.is_none())
    );
    assert!(work.steps > 0 && work.peak > 0 && work.peak <= MAX_BYTES);
    let artifact = &inputs.inventory.artifacts()[0];
    let owner = artifact.identity();
    let descriptor = artifact
        .source_descriptor()
        .ok_or("actual supplied descriptor")?;
    assert_eq!(descriptor.identity(), owner);
    assert_eq!(descriptor.provider(), owner);
    assert_eq!(
        descriptor.edition(),
        crate::support::edition::Edition::Rust2024
    );
    assert_eq!(descriptor.profile(), Profile::Source2024Provided);
    assert_eq!(
        inputs.inventory.source_provider_index(owner),
        Err("source client provider profile is unsupported")
    );
    Ok(())
}

#[test]
fn unpaid_reservations_reject_real_foreign_inventory_catalog_and_changed_cells() -> Result {
    let (report, _facts, inventory) = actual(true)?;
    let (other_report, _other_facts, other_inventory) = actual(true)?;
    let mut work = ReservationWork::default();
    let mut first = Inputs::reserve(report, inventory, 0, &mut work)?;
    let mut second = Inputs::reserve(other_report, other_inventory, 0, &mut work)?;
    assert!(first.require(&second.inventory, 0, &mut work).is_err());
    assert!(
        first
            .require(&first.inventory, u32::MAX, &mut work)
            .is_err()
    );
    std::mem::swap(&mut first.catalog, &mut second.catalog);
    assert!(first.require(&first.inventory, 0, &mut work).is_err());
    std::mem::swap(&mut first.catalog, &mut second.catalog);
    let function = first
        .definitions
        .clone()
        .find(|&index| {
            first.report.resolution.definitions[index].kind
                == crate::resolve::model::DefKind::Function
        })
        .ok_or("actual reserved function")?;
    first.report.resolution.definitions[function].kind = crate::resolve::model::DefKind::Trait;
    assert!(first.check_cells(&mut work).is_err());
    Ok(())
}

#[test]
fn unpaid_reservation_failures_keep_attempts_without_returning_an_input_owner() -> Result {
    let (report, _facts, inventory) = actual(false)?;
    assert!(
        inventory
            .source_provider_index(inventory.artifacts()[0].identity())
            .is_ok()
    );
    let mut ordinary = ReservationWork::default();
    assert!(Inputs::reserve(report, inventory, 0, &mut ordinary).is_err());
    assert!(ordinary.steps > 0);
    for allowance in [0, 4, 32, 128] {
        let (report, _facts, inventory) = actual(true)?;
        let mut work = ReservationWork {
            steps: MAX_COMPONENTS - allowance,
            ..ReservationWork::default()
        };
        assert!(Inputs::reserve(report, inventory, 0, &mut work).is_err());
        assert!(work.steps > MAX_COMPONENTS);
    }
    let (report, _facts, inventory) = actual(true)?;
    let mut storage = ReservationWork {
        retained: MAX_BYTES,
        ..ReservationWork::default()
    };
    assert!(Inputs::reserve(report, inventory, 0, &mut storage).is_err());
    assert!(storage.retained > MAX_BYTES && storage.peak > MAX_BYTES && storage.steps > 0);
    Ok(())
}
