//! Scope and owned-value coordinates stay explicit before normal storage reproof.
use super::{DomainIndex, Work, g, s};

impl DomainIndex<'_, '_> {
    pub(in super::super) fn storage_ports(
        &mut self,
        original: &s::Original,
        work: &mut Work,
    ) -> Result<(), &'static str> {
        self.scratch(
            original
                .statements
                .len()
                .saturating_add(original.values.len()),
            work,
        )?;
        Self::scopes(original, work)?;
        self.statements(original, work)?;
        for exit in &original.exits {
            Self::location(original, exit.location, work)?;
            match exit.end {
                g::End::Scopes { inner, outer }
                    if inner as usize >= original.scopes.len()
                        || outer.is_some_and(|outer| outer >= inner) =>
                {
                    return Err("source cleanup exit has foreign scope coordinates");
                }
                g::End::Temporaries(range) => {
                    range.slice(&original.temporaries)?;
                }
                g::End::Scopes { .. } => {}
            }
        }
        for temporary in &original.temporaries {
            Self::temporary(original, temporary, work)?;
        }
        if let Some(owned) = &original.owned {
            work.charge(original.values.len())?;
            let mut created = vec![false; original.values.len()];
            for temporary in &owned.created {
                Self::temporary(original, temporary, work)?;
                if std::mem::replace(&mut created[temporary.value as usize], true) {
                    return Err("source owned rvalue has duplicate creation coordinates");
                }
            }
            for release in &owned.returned {
                Self::location(original, release.location, work)?;
                if created.get(release.value as usize) != Some(&true)
                    || release.exits_before as usize > original.exits.len()
                {
                    return Err("source owned return has no exact creation/exit domain");
                }
                let g::Location::At { block, .. } = release.location else {
                    return Err("source owned return requires its actual return block");
                };
                if original.blocks[block as usize].terminator
                    != g::Terminator::Return(release.value)
                {
                    return Err("source owned return belongs to another actual terminator value");
                }
            }
        }
        Ok(())
    }
    fn scopes(original: &s::Original, work: &mut Work) -> Result<(), &'static str> {
        if original.scopes.is_empty()
            || original.scopes[0].parent.is_some()
            || original.scopes[0].kind != g::ScopeKind::Body
        {
            return Err("source original lost its genuine body scope");
        }
        for (index, scope) in original.scopes.iter().enumerate() {
            Self::site(original, scope.site, work)?;
            if index > 0 && scope.parent.is_none_or(|parent| parent as usize >= index) {
                return Err("source lexical scopes are forward, cyclic or multiply rooted");
            }
        }
        for local in &original.locals {
            work.charge(1)?;
            if local.scope as usize >= original.scopes.len()
                || local
                    .storage_parent
                    .is_some_and(|parent| parent as usize >= original.locals.len())
            {
                return Err("source local storage has foreign scope or parent coordinates");
            }
        }
        Ok(())
    }
    fn statements(&mut self, original: &s::Original, work: &mut Work) -> Result<(), &'static str> {
        for statement in &original.statements {
            Self::site(original, statement.site, work)?;
            self.observed.operation_ports = self.observed.operation_ports.saturating_add(1);
            match statement.action {
                s::Action::StorageLive(local) | s::Action::StorageDead(local) => {
                    if local as usize >= original.locals.len() {
                        return Err("source storage local is unknown");
                    }
                }
                s::Action::StorageEnd { first, last } => {
                    if first > last || last as usize >= original.locals.len() {
                        return Err("source storage suffix is reversed or foreign");
                    }
                }
                s::Action::Project(projection) => {
                    Self::place(original, s::Place::Projection(projection), None, work)?;
                }
                s::Action::Evaluate(value) => {
                    if value as usize >= original.values.len() {
                        return Err("source evaluated value is unknown");
                    }
                }
                s::Action::Store { place, value } => {
                    Self::place(original, place, None, work)?;
                    if value as usize >= original.values.len() {
                        return Err("source stored value is unknown");
                    }
                }
                s::Action::CheckInitialized(place) | s::Action::Mention(place) => {
                    Self::place(original, place, None, work)?;
                }
            }
        }
        work.charge(original.statements.len())?;
        let mut covered = vec![false; original.statements.len()];
        if original.blocks.is_empty() {
            return Err("source original has no actual entry block");
        }
        for block in &original.blocks {
            Self::site(original, block.site, work)?;
            block.statements.slice(&original.statements)?;
            for index in block.statements.start..block.statements.end {
                work.charge(1)?;
                if std::mem::replace(&mut covered[index as usize], true) {
                    return Err("source CFG blocks overlap actual statement ownership");
                }
            }
            match block.terminator {
                g::Terminator::Return(value) if value as usize >= original.values.len() => {
                    return Err("source return value is unknown");
                }
                g::Terminator::Jump(target) if target as usize >= original.blocks.len() => {
                    return Err("source CFG successor is foreign");
                }
                _ => {}
            }
        }
        work.charge(covered.len())?;
        if covered.contains(&false) {
            return Err("source CFG statements have incomplete block coverage");
        }
        Ok(())
    }
    fn location(
        original: &s::Original,
        location: g::Location,
        work: &mut Work,
    ) -> Result<(), &'static str> {
        work.charge(1)?;
        let valid = match location {
            g::Location::At { block, before } => {
                original.blocks.get(block as usize).is_some_and(|block| {
                    before >= block.statements.start && before <= block.statements.end
                })
            }
            g::Location::Edge { block, target } => original
                .blocks
                .get(block as usize)
                .is_some_and(|block| block.terminator == g::Terminator::Jump(target)),
        };
        if !valid {
            return Err("source storage location is foreign to its actual CFG");
        }
        Ok(())
    }
    fn temporary(
        original: &s::Original,
        temporary: &g::Temporary,
        work: &mut Work,
    ) -> Result<(), &'static str> {
        Self::site(original, temporary.site, work)?;
        Self::location(
            original,
            g::Location::At {
                block: temporary.block,
                before: temporary.before,
            },
            work,
        )?;
        if temporary.value as usize >= original.values.len()
            || temporary.exits_before as usize > original.exits.len()
            || matches!(temporary.lifetime, g::Lifetime::Scope(scope) if scope as usize >= original.scopes.len())
        {
            return Err("source temporary has foreign value/scope/exit coordinates");
        }
        Ok(())
    }
}
