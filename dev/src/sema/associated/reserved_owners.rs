//! Install complete authenticated owner rows without inventing source declarations.
use super::{Inventory, Member, Owner, OwnerId, OwnerKind, Schema};
use crate::{resolve::model::DefId, sema::crates::ImportedOwners, support::hash::HashSet};

pub(in crate::sema) struct ImportedOwner {
    pub owner: Owner,
    pub schema: Schema,
    pub members: Vec<DefId>,
}
#[cfg(test)]
pub(in crate::sema) fn imported_layouts() -> (usize, usize, usize) {
    (
        size_of::<super::language::Declarations>(),
        size_of::<Inventory>(),
        size_of::<Option<ImportedOwners>>(),
    )
}
impl Inventory {
    pub(in crate::sema) fn imported_allocation_bound(
        &self,
        owners: usize,
        members: usize,
    ) -> usize {
        use crate::support::retained::new_map;
        let declarations = owners.saturating_mul(2).saturating_add(members);
        4_usize
            .saturating_mul(self.owners.len().saturating_add(owners))
            .saturating_mul(size_of::<Owner>() + size_of::<Vec<DefId>>())
            .saturating_add(
                4_usize
                    .saturating_mul(self.members.len().saturating_add(members))
                    .saturating_mul(size_of::<Member>()),
            )
            .saturating_add(new_map::<DefId, OwnerId>(
                self.declarations.len().saturating_add(declarations),
            ))
            .saturating_add(new_map::<OwnerId, Schema>(
                self.schemas.len().saturating_add(owners),
            ))
            .saturating_add(size_of::<super::language::Declarations>())
            .saturating_add(new_map::<DefId, ()>(declarations))
    }
    pub(in crate::sema) fn imported_table_allocation(
        &self,
    ) -> crate::support::retained::Allocation {
        let mut result = crate::support::retained::Allocation::default();
        result.vector(&self.owners);
        result.vector(&self.members);
        result.vector(&self.owner_members);
        result.map(&self.declarations);
        result.map(&self.schemas);
        if let Some(language) = &self.language {
            result.boxed(language.as_ref());
        }
        result
    }
    /// The caller reserves the complete destination/scratch capacity in its one
    /// import budget before entering this operation. All checks precede writes.
    pub(in crate::sema) fn install_imported(
        &mut self,
        store: &crate::types::Store,
        graph: ImportedOwners,
        rows: Vec<ImportedOwner>,
        steps: &mut usize,
    ) -> Result<(), &'static str> {
        *steps = steps.saturating_add(1);
        graph.require_store(store)?;
        if rows.len() != graph.associated_count()?
            || self
                .language
                .as_ref()
                .is_some_and(|language| !language.accepts_imported())
        {
            return Err(
                "imported owner inventory is incomplete or would coexist with bootstrap declarations",
            );
        }
        self.append_rows(store, graph, rows, steps)
    }
    pub(in crate::sema) fn imported_owner(&self) -> Option<&ImportedOwners> {
        self.language.as_ref()?.imported.as_ref()
    }
    pub(in crate::sema) fn selected_allocation_bound(
        &self,
        members: usize,
    ) -> crate::support::retained::Growth {
        use crate::support::retained::Growth;
        let declarations = members.saturating_add(2);
        let mut growth = Growth::vector::<Owner>(self.owners.len(), self.owners.capacity(), 1);
        growth.include(Growth::vector::<Vec<DefId>>(
            self.owner_members.len(),
            self.owner_members.capacity(),
            1,
        ));
        growth.include(Growth::vector::<Member>(
            self.members.len(),
            self.members.capacity(),
            members,
        ));
        growth.include(Growth::map::<DefId, OwnerId>(
            self.declarations.len(),
            self.declarations.capacity(),
            declarations,
        ));
        growth.include(Growth::map::<OwnerId, Schema>(
            self.schemas.len(),
            self.schemas.capacity(),
            1,
        ));
        growth.include(Growth::map::<DefId, ()>(0, 0, declarations));
        if self.language.is_none() {
            growth.bytes = growth
                .bytes
                .saturating_add(size_of::<super::language::Declarations>());
        }
        growth
    }
    pub(in crate::sema) fn append_selected(
        &mut self,
        store: &crate::types::Store,
        mapping: ImportedOwners,
        row: ImportedOwner,
        steps: &mut usize,
    ) -> Result<OwnerId, &'static str> {
        *steps = steps.saturating_add(1);
        mapping.require_store(store)?;
        if !matches!(mapping, ImportedOwners::Provided(_))
            || self
                .language
                .as_ref()
                .is_some_and(|language| match language.imported.as_ref() {
                    Some(actual) => !actual.same_owner(&mapping),
                    None => !language.accepts_imported(),
                })
        {
            return Err("selected imported append changed its actual mapping or bootstrap owner");
        }
        let id = OwnerId(
            u32::try_from(self.owners.len())
                .map_err(|_| "imported owner identity space exhausted")?,
        );
        self.append_rows(store, mapping, [row], steps)?;
        Ok(id)
    }
    fn append_rows<R: AsRef<[ImportedOwner]> + IntoIterator<Item = ImportedOwner>>(
        &mut self,
        store: &crate::types::Store,
        graph: ImportedOwners,
        rows: R,
        steps: &mut usize,
    ) -> Result<(), &'static str> {
        let end = self
            .owners
            .len()
            .checked_add(rows.as_ref().len())
            .ok_or("imported owner identity count overflow")?;
        u32::try_from(end).map_err(|_| "imported owner identity space exhausted")?;
        let mut owner_index = u32::try_from(self.owners.len())
            .map_err(|_| "imported owner identity space exhausted")?;
        let mut identities = HashSet::default();
        for row in rows.as_ref() {
            graph.associated_row(store, row.owner, &row.schema, &row.members, steps)?;
            let definitions = std::iter::once(row.owner.self_definition)
                .chain(match row.owner.kind {
                    OwnerKind::Trait { definition, .. } => Some(definition),
                    OwnerKind::ImportedImpl(_) | OwnerKind::Impl { .. } => None,
                })
                .chain(row.members.iter().copied());
            for definition in definitions {
                *steps = steps.saturating_add(1);
                if !identities.insert(definition) || self.declarations.contains_key(&definition) {
                    return Err(
                        "imported associated declaration is duplicate or already installed",
                    );
                }
            }
        }
        if *steps > crate::support::crates::MAX_COMPONENTS {
            return Err("imported owner installation work budget exceeded");
        }
        // No fallible operation follows the first write. These IDs were bounded
        // together above, and the exclusive borrow retains the preflight state.
        for row in rows {
            let id = OwnerId(owner_index);
            owner_index += 1;
            self.declarations.insert(row.owner.self_definition, id);
            if let OwnerKind::Trait { definition, .. } = row.owner.kind {
                self.declarations.insert(definition, id);
            }
            for &definition in &row.members {
                self.declarations.insert(definition, id);
                self.members.push(Member {
                    definition,
                    owner: id,
                });
            }
            self.owners.push(row.owner);
            self.owner_members.push(row.members);
            self.schemas.insert(id, row.schema);
        }
        self.language.get_or_insert_default().imported = Some(graph);
        Ok(())
    }
}
