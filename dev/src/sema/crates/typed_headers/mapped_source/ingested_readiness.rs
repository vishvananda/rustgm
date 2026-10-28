//! Declaration readiness checks installed data; original bodies remain unproved.
use super::{Catalog, DefId, Graph, SignaturePort, s, slot_bindings};
use crate::{
    sema::model::Declarations,
    types::{ItemId, Kind},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::sema) enum Readiness {
    Declaration,
    ValueBinding,
}
impl Graph {
    pub(in crate::sema::crates::typed_headers) fn readiness(
        &self,
        catalog: &Catalog,
        facts: &Declarations,
        headers: &super::super::Headers,
        definition: DefId,
        steps: &mut usize,
    ) -> Result<Readiness, &'static str> {
        *steps = steps.saturating_add(1);
        self.require_store(&facts.types)?;
        let descriptor = self.descriptor(catalog)?;
        let id = catalog.declaration(definition)?;
        let (owner, row, _) = catalog.declaration_coordinate(id)?;
        if owner != descriptor.identity() {
            return Err("source declaration readiness belongs to a foreign artifact");
        }
        let rows = descriptor.rows();
        let raw = rows
            .declarations
            .get(row.0 as usize)
            .ok_or("source readiness declaration is outside its actual table")?;
        catalog.validate_declaration(id, definition, slot_bindings::kind(raw.definition.kind))?;
        let cell = facts
            .definitions
            .get(definition.0 as usize)
            .ok_or("source readiness declaration is outside its semantic cells")?;
        match raw.definition.kind {
            s::DefinitionKind::Module => {
                if cell.is_some() || rows.header(row)?.is_some() {
                    return Err("source module readiness requires its actual typeless module");
                }
                if descriptor.profile().module_contracts() {
                    descriptor.module_contracts()?.require_module(row)?;
                } else if row != self.root_row {
                    return Err("legacy source module readiness requires its actual root");
                }
            }
            s::DefinitionKind::Trait => {
                if cell.is_some() {
                    return Err("source trait readiness cannot invent a value type");
                }
                self.trait_ready(catalog, facts, definition, row, steps)?;
            }
            s::DefinitionKind::Function => {
                self.function_ready(catalog, facts, headers, definition, steps)?;
            }
            s::DefinitionKind::SelfType => {
                let owner = self.installed_owner(facts, definition)?;
                let schema = facts
                    .associated
                    .schema(definition)
                    .ok_or("source Self readiness lacks its actual completed schema")?;
                if owner.self_definition != definition || *cell != Some(schema.self_type) {
                    return Err("source Self readiness changed its actual owner/type cell");
                }
                Self::scope_ready(facts, definition)?;
            }
            s::DefinitionKind::TypeParameter => {
                let parent = raw
                    .definition
                    .parent
                    .ok_or("source formal readiness lacks its actual owner")?;
                Self::formal_ready(
                    facts,
                    definition,
                    catalog.definition_at(owner, parent)?,
                    *cell,
                )?;
            }
            s::DefinitionKind::ValueParameter | s::DefinitionKind::Local => {
                if cell.is_some() || rows.header(row)?.is_some() {
                    return Err("source body-local readiness cannot install declaration types");
                }
                Self::value_binding_ready(raw, rows)?;
                return Ok(Readiness::ValueBinding);
            }
            s::DefinitionKind::RegionParameter => {
                return Err("source named lifetime readiness requires its wider callable profile");
            }
        }
        Ok(Readiness::Declaration)
    }

    fn value_binding_ready(
        binding: &s::Declaration,
        rows: s::ValidatedView<'_>,
    ) -> Result<(), &'static str> {
        if binding.local.is_some() {
            return Ok(());
        }
        let parent = binding
            .definition
            .parent
            .ok_or("source value binding lacks its actual function parent")?;
        let Some(s::HeaderRef::Function(index)) = rows.header(parent)? else {
            return Err("source value binding lacks its actual function header");
        };
        // The sealed formal ledger established unique membership once. Only a
        // required method's genuine formal may lack original-local coverage.
        if binding.definition.kind != s::DefinitionKind::ValueParameter
            || rows.functions[index as usize].body != s::FunctionBody::Absent
        {
            return Err("source provided value binding omitted its actual original local");
        }
        Ok(())
    }

    fn formal_ready(
        facts: &Declarations,
        definition: DefId,
        owner: DefId,
        cell: Option<crate::types::TypeId>,
    ) -> Result<(), &'static str> {
        let Some(Kind::Parameter(parameter)) = cell.and_then(|ty| facts.types.kind(ty)) else {
            return Err("source formal readiness lacks its actual parameter cell");
        };
        let actual = facts
            .types
            .parameter(parameter)
            .ok_or("source formal readiness lost its actual parameter identity")?;
        let formal = facts
            .generics
            .parameters
            .get(&owner)
            .and_then(|parameters| parameters.get(actual.index as usize))
            .ok_or("source formal readiness omitted its actual original slot")?;
        if actual.owner != ItemId(owner.0)
            || formal.binding != definition
            || Some(formal.argument) != cell.map(crate::types::arguments::Argument::Type)
        {
            return Err("source formal readiness changed its actual parameter owner/slot");
        }
        Ok(())
    }

    fn installed_owner(
        &self,
        facts: &Declarations,
        definition: DefId,
    ) -> Result<crate::sema::associated::Owner, &'static str> {
        let owner = facts
            .associated
            .owner(definition)
            .and_then(|id| facts.associated.get(id))
            .copied()
            .ok_or("source declaration readiness lacks its actual installed owner")?;
        facts.associated.require_imported(owner)?;
        self.owner(owner)?;
        Ok(owner)
    }

    pub(super) fn trait_ready(
        &self,
        catalog: &Catalog,
        facts: &Declarations,
        definition: DefId,
        row: crate::support::crates::owner_catalog::DeclarationRef,
        steps: &mut usize,
    ) -> Result<(), &'static str> {
        let owner = self.installed_owner(facts, definition)?;
        let schema = facts
            .associated
            .schema(definition)
            .ok_or("source trait readiness lacks its completed schema")?;
        let members = facts
            .associated
            .owner(definition)
            .and_then(|id| facts.associated.owner_members(id))
            .ok_or("source trait readiness lacks its actual member inventory")?;
        self.associated_row(&facts.types, owner, schema, members, steps)?;
        let header = facts
            .generics
            .traits
            .as_deref()
            .and_then(|contracts| contracts.header(ItemId(definition.0)))
            .ok_or("source trait readiness lacks its installed canonical input contract")?;
        header.inputs.require_catalog(catalog, definition)?;
        let rows = self.descriptor(catalog)?.rows();
        let Some(s::HeaderRef::Trait(index)) = rows.header(row)? else {
            return Err("source trait readiness changed its actual header kind");
        };
        let raw = &rows.traits[index as usize];
        let full = self
            .trait_inputs
            .get(index as usize)
            .and_then(Option::as_deref)
            .ok_or("source trait readiness lacks its complete remapped inputs")?;
        let expected = full
            .get(..raw.canonical_inputs as usize)
            .ok_or("source trait readiness truncated its canonical input prefix")?;
        let inputs = header.inputs.view();
        if header.self_type != schema.self_type || inputs.len() != expected.len() {
            return Err("source trait readiness changed its actual Self or canonical prefix");
        }
        for (actual, expected) in inputs.iter().zip(expected) {
            *steps = steps.saturating_add(1);
            if actual.source != expected.source || actual.value != expected.value {
                return Err("source trait readiness changed a remapped canonical predicate");
            }
        }
        Self::scope_ready(facts, definition)
    }

    pub(super) fn function_ready(
        &self,
        catalog: &Catalog,
        facts: &Declarations,
        headers: &super::super::Headers,
        definition: DefId,
        steps: &mut usize,
    ) -> Result<(), &'static str> {
        let header = headers
            .get(definition)
            .ok_or("source function readiness lacks its actual canonical header")?;
        facts
            .function_header(definition)?
            .ok_or("source function readiness lacks its actual declaration header")?;
        if !self.function(catalog, definition, header.body, header.universals)? {
            return Err("source function readiness changed its actual defining artifact");
        }
        let id = catalog.declaration(definition)?;
        let (_, row, _) = catalog.declaration_coordinate(id)?;
        let rows = self.descriptor(catalog)?.rows();
        let Some(s::HeaderRef::Function(index)) = rows.header(row)? else {
            return Err("source function readiness changed its actual header kind");
        };
        let raw = &rows.functions[index as usize];
        self.require_signature(
            catalog,
            &facts.types,
            SignaturePort {
                owner: row,
                kind: s::BinderKind::OriginalFunction,
                raw: raw.original,
            },
            header.signature,
            steps,
        )?;
        self.require_signature(
            catalog,
            &facts.types,
            SignaturePort {
                owner: row,
                kind: s::BinderKind::PublicFunction,
                raw: raw.callable,
            },
            header.callable,
            steps,
        )?;
        let scheme = facts
            .lifetimes
            .functions
            .get(&definition)
            .ok_or("source function readiness omitted its actual lifetime scheme")?;
        self.require_method_regions(catalog, definition, scheme, steps)?;
        Self::environment_ready(facts, definition, header, raw, rows)?;
        Self::scope_ready(facts, definition)
    }

    pub(in crate::sema::crates::typed_headers) fn require_method_environment(
        &self,
        catalog: &Catalog,
        facts: &Declarations,
        definition: DefId,
        header: &super::super::Header,
        steps: &mut usize,
    ) -> Result<(), &'static str> {
        *steps = steps.saturating_add(9);
        self.require_store(&facts.types)?;
        let descriptor = self.descriptor(catalog)?;
        let declaration = catalog.declaration(definition)?;
        catalog.validate_declaration(
            declaration,
            definition,
            crate::resolve::model::DefKind::Function,
        )?;
        let (owner, row, _) = catalog.declaration_coordinate(declaration)?;
        if owner != descriptor.identity()
            || header.declaration
                != crate::resolve::body_origin::DeclarationOrigin::Imported(declaration)
            || !self.function(catalog, definition, header.body, header.universals)?
        {
            return Err("imported method environment changed its actual declaration/body owner");
        }
        let rows = descriptor.rows();
        let Some(s::HeaderRef::Function(index)) = rows.header(row)? else {
            return Err("imported method environment lacks its actual function row");
        };
        Self::environment_ready(
            facts,
            definition,
            header,
            &rows.functions[index as usize],
            rows,
        )
    }

    fn environment_ready(
        facts: &Declarations,
        definition: DefId,
        header: &super::super::Header,
        raw: &s::FunctionHeader,
        rows: s::ValidatedView<'_>,
    ) -> Result<(), &'static str> {
        let environment = facts
            .generics
            .environments
            .get(&definition)
            .ok_or("source function readiness omitted its actual declaration environment")?;
        let defining = header
            .environment
            .as_ref()
            .ok_or("source method requires its actual pending defining environment")?;
        match raw.body {
            s::FunctionBody::Absent if !std::sync::Arc::ptr_eq(environment, defining) => {
                return Err("required source method changed its actual declaration environment");
            }
            s::FunctionBody::Original(body) => {
                let input = rows
                    .body_inputs
                    .get(body.0 as usize)
                    .filter(|input| input.body == body)
                    .ok_or("source function readiness omitted its actual original input owner")?;
                match input.environment {
                    s::EnvironmentOrigin::Declaration(owner) if owner == raw.declaration => {
                        if !std::sync::Arc::ptr_eq(environment, defining) {
                            return Err(
                                "source function changed its actual declaration environment",
                            );
                        }
                    }
                    s::EnvironmentOrigin::Contract
                        if !std::sync::Arc::ptr_eq(environment, defining) => {}
                    _ => {
                        return Err(
                            "source function changed its actual original environment provenance",
                        );
                    }
                }
            }
            s::FunctionBody::Absent => {}
        }
        Ok(())
    }

    fn scope_ready(facts: &Declarations, definition: DefId) -> Result<(), &'static str> {
        if !facts.generics.parameters.contains_key(&definition)
            || !facts.generics.environments.contains_key(&definition)
        {
            return Err("source readiness omitted its actual full formal/input scope");
        }
        Ok(())
    }
}
