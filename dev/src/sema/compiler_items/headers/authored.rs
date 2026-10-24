//! Only this adapter reads source syntax; validators consume borrowed typed facts.
use super::{Flags, Trait, Written};
use crate::{
    resolve::{bindings::Declaration, expansion::Report, model::DefKind},
    sema::{associated::OwnerKind, declaration_inputs, model::Declarations},
    syntax::{ast::Kind, item::ItemKind},
    types::arguments::Argument,
};

impl<'a> Trait<'a> {
    pub(in crate::sema) fn source(
        report: &Report,
        facts: &'a Declarations,
        declaration: Declaration,
    ) -> Result<Self, &'static str> {
        Self::source_with_bounds(report, facts, declaration).map(|(header, _)| header)
    }

    /// Borrow the actual literal bound nodes alongside the validated typed view.
    /// The source and semantic borrows keep their independent owners/lifetimes.
    pub(in crate::sema) fn source_with_bounds<'r>(
        report: &'r Report,
        facts: &'a Declarations,
        declaration: Declaration,
    ) -> Result<(Self, &'r [crate::syntax::ast::NodeId]), &'static str> {
        let definition = report
            .resolution
            .definitions
            .get(declaration.definition.0 as usize)
            .ok_or("source trait identity has no declaration")?;
        if definition.kind != DefKind::Trait || definition.node != Some(declaration.origin) {
            return Err("source trait identity does not match its actual declaration");
        }
        let ast = &report.package.unit(declaration.origin.file).configured.ast;
        let Kind::Item(index) = ast.node(declaration.origin.node).kind else {
            return Err("source trait identity has no item owner");
        };
        let item = ast.items[index as usize];
        let ItemKind::Trait {
            bounds,
            auto_: written_auto,
            unsafe_: written_unsafe,
            constant: written_const,
            ..
        } = item.kind
        else {
            return Err("source trait identity has no trait syntax");
        };
        let owner_id = facts
            .associated
            .owner(declaration.definition)
            .ok_or("source trait role has no prepared owner")?;
        let owner = facts
            .associated
            .get(owner_id)
            .ok_or("missing trait owner")?;
        let OwnerKind::Trait {
            definition: actual,
            unsafe_,
            constant,
            auto_,
        } = owner.kind
        else {
            return Err("source trait role belongs to an implementation");
        };
        if actual != declaration.definition
            || owner.source != declaration.origin
            || (auto_, unsafe_, constant) != (written_auto, written_unsafe, written_const)
        {
            return Err("source trait owner changed its declaration");
        }
        let schema = facts
            .associated
            .schema(declaration.definition)
            .ok_or("source trait role has no completed schema")?;
        let header = Self {
            definition: declaration.definition,
            parent: declaration.parent,
            origin: declaration.origin.into(),
            self_definition: owner.self_definition,
            self_type: schema.self_type,
            flags: Flags {
                unsafe_,
                constant,
                auto_,
            },
            attributes: facts.source_bindings.as_deref().map_or(
                Ok(crate::resolve::bindings::HeaderAttributes::EMPTY),
                |catalog| catalog.neutral_attributes(super::super::Role::from(declaration)),
            )?,
            captures: schema.captures,
            parameters: facts
                .generics
                .parameters
                .get(&actual)
                .ok_or("source trait role has no parameter contract")?,
            members: facts
                .associated
                .owner_members(owner_id)
                .ok_or("source trait role has no completed members")?,
            inputs: declaration_inputs::own_view(&facts.generics, facts.crates.as_deref(), actual)?,
            written: Written {
                parameters: ast.list(item.generics).len(),
                supertraits: ast.list(bounds).len(),
            },
        };
        header.validate_self(&facts.types)?;
        validate_binding(report, facts, &header)?;
        Ok((header, ast.list(bounds)))
    }
}

fn validate_binding(
    report: &Report,
    facts: &Declarations,
    header: &Trait<'_>,
) -> Result<(), &'static str> {
    let definition = report
        .resolution
        .definitions
        .get(header.definition.0 as usize)
        .ok_or("source trait declaration disappeared")?;
    if header.origin.source() != definition.node
        || report
            .resolution
            .definitions
            .get(header.self_definition.0 as usize)
            .is_none_or(|self_definition| {
                self_definition.kind != DefKind::SelfType
                    || definition.members != Some(self_definition.scope)
            })
        || facts.generics.parents.get(&header.definition) != Some(&header.self_definition)
    {
        return Err("source trait Self declaration has a different owner");
    }
    let parameters = facts
        .generics
        .parameters
        .get(&header.self_definition)
        .ok_or("source trait has no Self binding")?;
    let [parameter] = parameters.as_slice() else {
        return Err("source trait must retain exactly one Self binding");
    };
    if parameter.binding != header.self_definition
        || parameter.anonymous
        || parameter.default.is_some()
        || parameter.argument != Argument::Type(header.self_type)
    {
        return Err("source trait changed its original Self binding");
    }
    Ok(())
}
