//! Reserve all alias identities and formals before compiling any dependency edge.
use super::{Checker, Error, Predicate, regions};

impl Checker<'_> {
    /// Alias declarations retain shared edges; other owners receive real clauses.
    pub(in crate::sema::declarations) fn complete_alias_bound(
        &mut self,
        owner: super::DefId,
        edge: super::Edge,
        completed: &mut Vec<Predicate>,
        dependencies: &mut Vec<super::Edge>,
    ) -> Result<(), Error> {
        if self.alias_self(owner).is_some() {
            return self.push_alias_edge(dependencies, edge);
        }
        if edge.original_subject != edge.subject
            || self.facts.unreduced.arguments(edge.source).is_some()
        {
            return Err(self.error(
                edge.source,
                "unreduced alias inputs require complete projection consumers",
            ));
        }
        let clauses = self.expand_alias_edge(edge)?;
        self.charge_alias_growth(crate::support::retained::Growth::vector::<Predicate>(
            completed.len(), completed.capacity(), clauses.len(),
        ))?;
        completed.extend(clauses);
        Ok(())
    }

    pub(in crate::sema::declarations) fn reserve_alias_owners(&mut self) -> Result<(), Error> {
        let Some(mut pending) = self.trait_aliases.take() else {
            return Ok(());
        };
        let result = (|| {
            for header in &mut pending.headers {
                let owner = header.owner;
                self.reserve_lexical_self(
                    owner.source,
                    owner.definition,
                    owner.self_definition,
                    |_, value| {
                        if header.schema.is_some() {
                            return Err("trait alias schema is already installed");
                        }
                        // Install on the SAME canonical owner before Self completes.
                        header.schema = Some(value);
                        Ok(())
                    },
                )?;
            }
            Ok(())
        })();
        self.trait_aliases = Some(pending);
        result
    }
    pub(in crate::sema::declarations) fn prepare_alias_parameters(
        &mut self,
        steps: &mut usize,
        limit: usize,
    ) -> Result<(), Error> {
        let count = self
            .trait_aliases
            .as_ref()
            .map_or(0, |table| table.headers.len());
        for index in 0..count {
            let owner = self
                .trait_aliases
                .as_ref()
                .ok_or_else(|| Error::unlocated("missing alias parameters"))?
                .headers[index]
                .owner;
            let context = regions::Context {
                binder: Some(self.owner_binders[&owner.self_definition]),
                position: regions::Position::Output,
            };
            self.declaration_parameters(owner.definition, owner.source, context)?;
            self.facts
                .generics
                .parameters
                .entry(owner.definition)
                .or_default();
            self.drain(steps, limit)?;
        }
        Ok(())
    }
    pub(in crate::sema::declarations) fn complete_alias_headers(
        &mut self,
        steps: &mut usize,
        limit: usize,
    ) -> Result<(), Error> {
        let count = self
            .trait_aliases
            .as_ref()
            .map_or(0, |table| table.headers.len());
        for index in 0..count {
            let header = &mut self
                .trait_aliases
                .as_mut()
                .ok_or_else(|| Error::unlocated("missing alias contract owner"))?
                .headers[index];
            if std::mem::replace(&mut header.started, true) {
                return Err(Error::unlocated(
                    "trait alias contract preparation cannot restart",
                ));
            }
            let owner = header.owner;
            let context = regions::Context {
                binder: Some(self.owner_binders[&owner.self_definition]),
                position: regions::Position::Output,
            };
            self.schedule_bounds(owner.definition, context)?;
            self.drain(steps, limit)?;
            let clauses: Vec<Predicate> = self
                .facts
                .generics
                .predicates
                .remove(&owner.definition)
                .unwrap_or_default();
            self.trait_aliases
                .as_mut()
                .ok_or_else(|| Error::unlocated("missing alias contract table"))?
                .headers[index]
                .clauses = Some(clauses);
        }
        if let Some(pending) = &mut self.trait_aliases {
            pending.close().map_err(Error::unlocated)?;
        }
        Ok(())
    }
}
