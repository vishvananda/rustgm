//! Borrow authenticated imported role headers without manufacturing syntax.
use super::{Catalog, Graph, s};
use crate::{
    resolve::body_origin::DeclarationOrigin,
    sema::{
        compiler_items::{
            Role,
            headers::{Flags, Trait, Written},
        },
        declaration_inputs,
        model::Declarations,
    },
    types::arguments::Argument,
};

impl Graph {
    pub(in crate::sema::crates::typed_headers) fn trait_header<'a>(
        &self,
        catalog: &Catalog,
        facts: &'a Declarations,
        role: Role,
        steps: &mut usize,
    ) -> Result<Trait<'a>, &'static str> {
        *steps = steps.saturating_add(1);
        self.require_store(&facts.types)?;
        self.role_origin(role)?;
        let DeclarationOrigin::Imported(id) = role.origin else {
            return Err("imported trait header requires its actual declaration origin");
        };
        let descriptor = self.descriptor(catalog)?;
        let (_, row, _) = catalog.declaration_coordinate(id)?;
        let rows = descriptor.rows();
        let Some(s::HeaderRef::Trait(index)) = rows.header(row)? else {
            return Err("imported role lacks its complete actual trait header");
        };
        let raw = &rows.traits[index as usize];
        self.trait_ready(catalog, facts, role.definition, row, steps)?;
        installed(facts, role, raw, rows)
    }
}

/// The same installed declaration adapter follows either actual mapping check.
pub(in crate::sema::crates::typed_headers) fn installed<'a>(
    facts: &'a Declarations,
    role: Role,
    raw: &s::TraitHeader,
    rows: s::ValidatedView<'_>,
) -> Result<Trait<'a>, &'static str> {
    let owner_id = facts
        .associated
        .owner(role.definition)
        .ok_or("imported trait header lacks its installed actual owner")?;
    let owner = facts
        .associated
        .get(owner_id)
        .copied()
        .ok_or("imported trait header has a stale owner identity")?;
    let schema = facts
        .associated
        .schema(role.definition)
        .ok_or("imported trait header lacks its installed actual schema")?;
    let members = facts
        .associated
        .owner_members(owner_id)
        .ok_or("imported trait header lacks its complete actual member inventory")?;
    let catalog = facts
        .source_bindings
        .as_deref()
        .ok_or("imported trait header lacks its actual role catalog")?;
    let header = Trait {
        definition: role.definition,
        parent: role.parent,
        origin: role.origin,
        self_definition: owner.self_definition,
        self_type: schema.self_type,
        flags: Flags {
            unsafe_: raw.flags.bits() & s::TraitFlags::UNSAFE != 0,
            constant: raw.flags.bits() & s::TraitFlags::CONST != 0,
            auto_: raw.flags.bits() & s::TraitFlags::AUTO != 0,
        },
        attributes: catalog.neutral_attributes(role)?,
        captures: schema.captures,
        parameters: facts
            .generics
            .parameters
            .get(&role.definition)
            .ok_or("imported trait header omitted its actual formal contract")?,
        members,
        inputs: declaration_inputs::own_view(
            &facts.generics,
            facts.crates.as_deref(),
            role.definition,
        )?,
        written: Written {
            parameters: raw.written_parameters as usize,
            supertraits: raw.written_supertraits.slice(&rows.clause_indices)?.len(),
        },
    };
    header.validate_self(&facts.types)?;
    validate_self_binding(facts, &header)?;
    Ok(header)
}

fn validate_self_binding(facts: &Declarations, header: &Trait<'_>) -> Result<(), &'static str> {
    if facts.generics.parents.get(&header.definition) != Some(&header.self_definition) {
        return Err("imported trait header changed its actual Self inheritance");
    }
    let parameters = facts
        .generics
        .parameters
        .get(&header.self_definition)
        .ok_or("imported trait header omitted its actual Self binding")?;
    let [parameter] = parameters.as_slice() else {
        return Err("imported trait header must retain exactly one actual Self binding");
    };
    if parameter.binding != header.self_definition
        || parameter.anonymous
        || parameter.default.is_some()
        || parameter.argument != Argument::Type(header.self_type)
    {
        return Err("imported trait header changed its actual Self formal contract");
    }
    Ok(())
}
