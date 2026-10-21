//! Nominal parameter schemas complete independently of defaults and field templates.
use super::{Checker, DefId, DefKind, Error, ItemId, ItemKind, NodeKey, State, Type, Work};
use crate::{resolve::unit::FileId, sema::representation};

impl Checker<'_> {
    pub(in super::super) fn reserve_nominals(&mut self) -> Result<(), Error> {
        let representations = representation::analyze(self.report)?;
        for index in 0..self.report.resolution.definitions.len() {
            let definition = &self.report.resolution.definitions[index];
            if !matches!(definition.kind, DefKind::Struct | DefKind::Enum) {
                continue;
            }
            if self
                .report
                .resolution
                .library_types
                .identify(DefId(u32::try_from(index).map_err(|_| {
                    Error::at(FileId(0), 0, "too many nominal definitions")
                })?))
                .is_some()
            {
                continue;
            }
            let key = definition
                .node
                .ok_or_else(|| Error::at(FileId(0), 0, "nominal declaration has no syntax"))?;
            if !matches!(
                self.item(key)?.kind,
                ItemKind::Struct { .. } | ItemKind::Enum { .. }
            ) {
                return Err(self.error(
                    key,
                    "union semantics require the unsafe language assignment",
                ));
            }
            let id = u32::try_from(index)
                .map_err(|_| self.error(key, "too many nominal declarations"))?;
            let attributes = representations.get(&key).copied().unwrap_or_default();
            self.reserve_fields(ItemId(id), key, attributes)?;
        }
        self.index_constructors()
    }
    pub(in super::super) fn complete_nominal_headers(
        &mut self,
        steps: &mut usize,
        limit: usize,
    ) -> Result<(), Error> {
        for index in 0..self.nominals.entries.len() {
            let owner = DefId(self.nominals.entries[index].item.0);
            self.work.push(Work::Definition(owner, false));
            self.drain(steps, limit)?;
        }
        Ok(())
    }
    pub(in super::super) fn nominal_header(
        &mut self,
        owner: DefId,
        key: NodeKey,
        finish: bool,
    ) -> Result<(), Error> {
        if !finish {
            if self.states[owner.0 as usize] == State::Active {
                return Err(self.error(key, "cycle in nominal parameter declarations"));
            }
            self.states[owner.0 as usize] = State::Active;
            let context = self.template_context(owner, key)?;
            if context.binder.is_some() {
                self.nominal_regions.insert(ItemId(owner.0), context);
            }
            self.work.push(Work::Definition(owner, true));
            self.declaration_parameters(owner, key, context)?;
            return Ok(());
        }
        let arguments = self.template_arguments(owner, key)?;
        let ty = self.intern(key, Type::Adt(ItemId(owner.0), arguments))?;
        // Type annotations require the complete identity/schema, not the fields.
        // Defaults stay lazy so explicit self applications can use this identity.
        self.complete(owner, key, ty)
    }
}

impl Checker<'_> {
    pub(in super::super) fn nominal_contract(
        &mut self,
        item: ItemId,
        finish: bool,
    ) -> Result<(), Error> {
        let index = self.nominals.index[&item];
        let entry = &self.nominals.entries[index];
        let (key, state) = (entry.key, entry.contract);
        if state == State::Complete {
            return Ok(());
        }
        let owner = DefId(item.0);
        if !finish {
            if state == State::Active {
                return Err(self.error(key, "cycle in nominal declaration requirements"));
            }
            self.nominals.entries[index].contract = State::Active;
            self.work.push(Work::NominalContract(item, true));
            self.schedule_bounds(owner, self.nominal_context(item))?;
            return Ok(());
        }
        let context = self.nominal_context(item);
        if self.prepare_bound_environment(owner, context, Work::NominalContract(item, true))? {
            return Ok(());
        }
        self.complete_bound_environment(owner, context)?;
        self.facts
            .generics
            .complete_environment(&mut self.facts.types, owner)
            .map_err(|e| self.error(key, e))?;
        self.facts.generics.has_nominal_requirements |=
            self.facts.generics.parameters.contains_key(&owner)
                || self.facts.generics.predicates.contains_key(&owner);
        self.facts.generics.nominal_contracts.insert(owner);
        self.nominals.entries[index].contract = State::Complete;
        Ok(())
    }
}
