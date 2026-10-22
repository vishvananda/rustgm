//! Only reserved canonical integer range owners receive latent library readiness.
use super::{DefId, Head, Identity, Pending};
use crate::{
    resolve::{expansion::Report, model::DefKind},
    types::ItemId,
};
pub(super) struct Heads {
    pub(super) nominal: ItemId,
    pub(super) owners: [DefId; 12],
}
impl Pending {
    pub(in super::super) fn install_ranges(&mut self, report: &Report) -> Result<(), &'static str> {
        let canonical = report
            .resolution
            .builtin_traits
            .ranges()
            .ok_or("missing canonical range identities")?;
        let iteration = report
            .resolution
            .builtin_traits
            .iteration()
            .ok_or("missing range Iterator identity")?;
        let nominal = ItemId(canonical.nominal.0);
        if report
            .resolution
            .library_types
            .get(crate::support::library_types::LibraryType::Range)
            .is_none_or(|n| n.definition != canonical.nominal)
        {
            return Err("range scheduling differs from canonical nominal");
        }
        let expected = Identity::User(ItemId(
            iteration
                .get(crate::support::iteration::Trait::Iterator)
                .definition
                .0,
        ));
        for declaration in canonical.implementations {
            if self.owners.get(&declaration.definition)
                != Some(&(expected, Some(Head::Nominal(nominal))))
                || report
                    .resolution
                    .definitions
                    .get(declaration.definition.0 as usize)
                    .is_none_or(|d| d.kind != DefKind::SelfType || d.node.is_some())
            {
                return Err("range scheduling owner differs from exact reserved header");
            }
        }
        let heads = self.collections.get_or_insert_with(|| {
            Box::new(super::collections::Heads {
                iterator: None,
                entries: Vec::new(),
                mutable_iterator: None,
                cloning: None,
                ranges: None,
            })
        });
        if heads.ranges.is_some() {
            return Err("range scheduling installed twice");
        }
        heads.ranges = Some(Box::new(Heads {
            nominal,
            owners: canonical.implementations.map(|d| d.definition),
        }));
        Ok(())
    }
}
