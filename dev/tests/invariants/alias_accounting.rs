//! Alias utility costs use a real source ledger; these inputs grant no body proof.
use super::*;
use crate::types::{
    RegionId,
    relations::{BodyAliases, RegionEqualities},
};

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
pub(crate) fn check(declaration: &Environment, aliases: &[RegionId]) -> Result {
    assert!(declaration.neutral_alias_finish_bound(0).is_ok());
    let mut empty = declaration.clone();
    // Deliberately malformed prestate: presence must not be inferred from size.
    empty.regions = Some(Arc::new(RegionEqualities::default()));
    assert!(empty.neutral_alias_finish_bound(0).is_err());
    assert!(
        !aliases.is_empty(),
        "a real source ledger must exercise table allocation"
    );
    let bound = BodyAliases::construction_bound(aliases.len())?;
    let actual = Arc::new(BodyAliases::new(aliases.to_vec())?);
    let mut allocation = actual.allocation();
    allocation.add(size_of::<BodyAliases>() + 2 * size_of::<usize>());
    allocation.add(actual.control_allocation());
    assert!(allocation.bytes <= bound.bytes);
    // This checks the utility's indexed-slot boundary without minting any IDs.
    assert!(BodyAliases::construction_bound(usize::from(u16::MAX) + 1).is_ok());
    assert!(BodyAliases::construction_bound(usize::from(u16::MAX) + 2).is_err());
    let mut ledger = declaration.clone().with_body_aliases(aliases.to_vec())?;
    assert!(ledger.neutral_alias_finish_bound(aliases.len()).is_err());
    let total = declaration.neutral_alias_finish_bound(aliases.len())?;
    let actual = ledger.neutral_alias_owned_from(declaration)?;
    assert!(actual.bytes <= total.bytes);
    // Empty regional storage remains present even after replacing an alias ledger.
    ledger.regions = empty.regions;
    assert!(ledger.neutral_alias_finish_bound(aliases.len()).is_err());
    eprintln!(
        "actual alias utility slots={} bound={} actual={} work={} aliases={} equations={} environment={}",
        aliases.len(),
        total.bytes,
        actual.bytes,
        total.work,
        size_of::<BodyAliases>(),
        size_of::<RegionEqualities>(),
        size_of::<Environment>()
    );
    Ok(())
}
