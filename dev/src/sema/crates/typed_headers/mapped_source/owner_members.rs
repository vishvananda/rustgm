//! Associated inventory rows must match the complete actual immutable header graph.
use super::{DefKind, Graph, s};
use crate::resolve::model::DefId;
use crate::sema::associated::{Owner, OwnerKind, Schema};

impl Graph {
    pub(in crate::sema) fn denies_implementation(
        &self,
        owner: Owner,
    ) -> Result<bool, &'static str> {
        self.owner(owner)?;
        let OwnerKind::Trait { definition, .. } = owner.kind else {
            return Err("imported implementation policy requires its actual trait owner");
        };
        // The checked descriptor admits only readable Functions: its authored
        // producer rejects implementation-domain clauses before any headers.
        // Unknown/Provided profiles cannot establish this Unrestricted invariant.
        let descriptor = self.descriptor(&self.origins)?;
        let id = self.origins.declaration(definition)?;
        let (_, row, _) = self.origins.declaration_coordinate(id)?;
        let rows = descriptor.rows();
        let Some(s::HeaderRef::Trait(index)) = rows.header(row)? else {
            return Err("imported implementation policy lacks its completed actual trait header");
        };
        Ok(rows.traits[index as usize].flags.bits() & s::TraitFlags::DENY_IMPL != 0)
    }
    pub(in crate::sema) fn implementation_request(
        &self,
        owner: Owner,
    ) -> Result<crate::types::parameters::predicates::Predicate, &'static str> {
        self.owner(owner)?;
        let OwnerKind::ImportedImpl(id) = owner.kind else {
            return Err("imported candidate reservation requires its actual impl owner");
        };
        let descriptor = self.descriptor(&self.origins)?;
        let (_, row, _) = self.origins.declaration_coordinate(id)?;
        let rows = descriptor.rows();
        let Some(s::HeaderRef::Implementation(index)) = rows.header(row)? else {
            return Err("imported candidate reservation lacks its complete actual header");
        };
        let application = rows.implementations[index as usize]
            .trait_application
            .ok_or("imported inherent reservation requires its wider candidate profile")?;
        let predicate = self.application(application)?;
        self.implementation(owner, predicate)?;
        Ok(predicate)
    }
    pub(in crate::sema) fn associated_count(&self) -> Result<usize, &'static str> {
        let rows = self.descriptor(&self.origins)?.rows();
        Ok(rows.traits.len().saturating_add(rows.implementations.len()))
    }
    pub(in crate::sema) fn associated_row(
        &self,
        owner: Owner,
        schema: &Schema,
        members: &[DefId],
        steps: &mut usize,
    ) -> Result<(), &'static str> {
        *steps = steps.saturating_add(1);
        self.owner(owner)?;
        let descriptor = self.descriptor(&self.origins)?;
        let rows = descriptor.rows();
        let declaration = match owner.kind {
            OwnerKind::Trait { definition, .. } => definition,
            OwnerKind::ImportedImpl(_) => owner.self_definition,
            OwnerKind::Impl { .. } => {
                return Err("imported associated inventory cannot carry source syntax");
            }
        };
        let id = self.origins.declaration(declaration)?;
        let (artifact, row, _) = self.origins.declaration_coordinate(id)?;
        let (self_type, captures, bounds, expected_members) = match rows.header(row)? {
            Some(s::HeaderRef::Trait(index)) => {
                let raw = &rows.traits[index as usize];
                (
                    raw.self_type,
                    raw.region_captures,
                    raw.region_bounds,
                    raw.members.slice(&rows.members)?,
                )
            }
            Some(s::HeaderRef::Implementation(index)) => {
                let raw = &rows.implementations[index as usize];
                if raw.members.start != raw.members.end {
                    return Err(
                        "imported impl members require their wider exact conformance profile",
                    );
                }
                (
                    raw.self_type,
                    raw.region_captures,
                    raw.region_bounds,
                    &[][..],
                )
            }
            _ => return Err("imported associated inventory lacks its actual owner header"),
        };
        if artifact != descriptor.identity()
            || schema.captures != captures
            || self
                .ty(self_type)
                .map_err(|_| "missing imported actual Self type")?
                != schema.self_type
        {
            return Err("imported associated schema changed its actual owner/Self/captures");
        }
        let bounds = bounds.slice(&rows.region_bounds)?;
        if bounds.len() != schema.bounds.len() || expected_members.len() != members.len() {
            return Err("imported associated schema omitted members or full region relations");
        }
        for (&raw, &actual) in expected_members.iter().zip(members) {
            *steps = steps.saturating_add(1);
            let expected = self.origins.definition_at(artifact, raw)?;
            let id = self.origins.declaration(actual)?;
            self.origins
                .validate_declaration(id, actual, DefKind::Function)?;
            if expected != actual
                || rows
                    .declarations
                    .get(raw.0 as usize)
                    .is_none_or(|member| member.definition.parent != Some(row))
            {
                return Err("imported associated member changed its exact lexical owner/identity");
            }
        }
        for (raw, &actual) in bounds.iter().zip(&schema.bounds) {
            *steps = steps.saturating_add(1);
            if (
                self.region(raw.longer)
                    .map_err(|_| "invalid imported schema bound")?,
                self.region(raw.shorter)
                    .map_err(|_| "invalid imported schema bound")?,
            ) != actual
            {
                return Err("imported associated schema changed its actual region relation");
            }
        }
        Ok(())
    }
}
