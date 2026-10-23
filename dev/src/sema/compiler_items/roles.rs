//! Source roles become usable only after their actual complete headers validate.
mod copying;
mod destructibility;
pub(crate) use destructibility::destruct_predicate;
mod invocation;
use super::Catalog;
use crate::{
    resolve::{
        bindings::Declaration,
        expansion::Report,
        model::{DefId, DefKind},
        package::Error,
    },
    sema::{
        associated::OwnerKind,
        model::{Declarations, Function},
    },
    support::{bindings::LangItem, call_traits::CallTrait},
    syntax::{ast::Kind as Syntax, item::ItemKind},
    types::{Kind, TypeId, parameters::predicates::Bound},
};

pub(super) struct Validated {
    roles: u32,
    pub(super) methods: [Option<DefId>; 3],
}
impl Catalog {
    /// Missing, reserved and completed roles are distinct states.
    pub(crate) fn validated_language(
        &self,
        role: LangItem,
    ) -> Result<Option<Declaration>, &'static str> {
        let Some(declaration) = self.language(role) else {
            return Ok(None);
        };
        if self
            .traits
            .as_ref()
            .is_none_or(|v| v.roles & (1 << role.index()) == 0)
        {
            return Err("source trait role requires completed declaration validation");
        }
        Ok(Some(declaration))
    }
}

pub(in crate::sema) fn validate(report: &Report, facts: &mut Declarations) -> Result<(), Error> {
    let Some(catalog) = facts.source_bindings.as_deref() else {
        return Ok(());
    };
    let mut complete = Validated {
        roles: 0,
        methods: [None; 3],
    };
    let mut callables = Vec::new();
    for role in [
        LangItem::Clone,
        LangItem::Copy,
        LangItem::LegacyReceiver,
        LangItem::Tuple,
        LangItem::FnOnce,
        LangItem::FnMut,
        LangItem::Fn,
        LangItem::Drop,
        LangItem::Destruct,
    ] {
        let Some(declaration) = catalog.language(role) else {
            continue;
        };
        let self_type = header(report, facts, declaration, role)?;
        match role {
            LangItem::Clone | LangItem::Copy | LangItem::LegacyReceiver => copying::validate(
                report,
                facts,
                catalog,
                declaration,
                role,
                self_type,
                &mut complete.roles,
            )?,
            LangItem::Tuple => validate_marker(report, facts, declaration)?,
            LangItem::Drop | LangItem::Destruct => {
                destructibility::validate(report, facts, catalog, declaration, role, self_type)?;
            }
            LangItem::FnOnce | LangItem::FnMut | LangItem::Fn => {
                let (kind, method) =
                    invocation::validate(report, facts, catalog, declaration, role, self_type)?;
                complete.methods[kind as usize] = Some(method);
                if kind == CallTrait::FnOnce {
                    complete.roles |= 1 << LangItem::FnOnceOutput.index();
                }
                callables.push((declaration, kind, method));
            }
            _ => {
                return Err(Declarations::error(
                    report,
                    declaration.origin,
                    "invalid source trait role",
                ));
            }
        }
        complete.roles |= 1 << role.index();
    }
    for role in [LangItem::CloneFn, LangItem::FnOnceOutput] {
        if let Some(declaration) = catalog.language(role)
            && complete.roles & (1 << role.index()) == 0
        {
            return Err(Declarations::error(
                report,
                declaration.origin,
                "source language member has no validated actual trait owner",
            ));
        }
    }
    if complete.roles == 0 {
        return Ok(());
    }
    if catalog.traits.is_some() {
        return Err(Error::at(
            crate::resolve::unit::FileId(0),
            0,
            "source roles were validated twice",
        ));
    }
    let output = catalog.language(LangItem::FnOnceOutput);
    facts
        .associated
        .register_source_callables(&callables, output)
        .map_err(|e| Error::at(crate::resolve::unit::FileId(0), 0, e))?;
    facts
        .source_bindings
        .as_deref_mut()
        .ok_or_else(|| {
            Error::at(
                crate::resolve::unit::FileId(0),
                0,
                "source catalog disappeared",
            )
        })?
        .traits = Some(Box::new(complete));
    Ok(())
}

fn header(
    report: &Report,
    facts: &Declarations,
    declaration: Declaration,
    role: LangItem,
) -> Result<TypeId, Error> {
    let error = |message| Declarations::error(report, declaration.origin, message);
    let definition = &report.resolution.definitions[declaration.definition.0 as usize];
    let owner = facts
        .associated
        .owner(declaration.definition)
        .and_then(|id| facts.associated.get(id))
        .ok_or_else(|| error("source trait role has no prepared owner"))?;
    let schema = facts
        .associated
        .schema(declaration.definition)
        .ok_or_else(|| error("source trait role has no completed schema"))?;
    let constant = matches!(
        role,
        LangItem::Clone
            | LangItem::FnOnce
            | LangItem::FnMut
            | LangItem::Fn
            | LangItem::Drop
            | LangItem::Destruct
    );
    if declaration.parent.is_some()
        || definition.node != Some(declaration.origin)
        || definition.kind != DefKind::Trait
        || owner.source != declaration.origin
        || schema.captures != 0
        || !matches!(owner.kind, OwnerKind::Trait { definition, unsafe_: false, constant: actual }
            if definition == declaration.definition && actual == constant)
    {
        return Err(error(
            "source trait header is outside the pinned provider profile",
        ));
    }
    let parameters = facts
        .generics
        .parameters
        .get(&declaration.definition)
        .ok_or_else(|| error("source trait role has no parameter contract"))?;
    let expected = usize::from(matches!(
        role,
        LangItem::FnOnce | LangItem::FnMut | LangItem::Fn
    ));
    if parameters.len() != expected
        || !matches!(facts.types.kind(schema.self_type), Some(Kind::Parameter(_)))
    {
        return Err(error(
            "source trait role has unsupported generic parameters",
        ));
    }
    Ok(schema.self_type)
}

fn function(facts: &Declarations, definition: DefId) -> Option<Function> {
    facts
        .function_indices
        .get(&definition)
        .and_then(|index| facts.functions.get(*index))
        .copied()
}

fn item(report: &Report, declaration: Declaration) -> Result<crate::syntax::item::Item, Error> {
    let unit = report.package.unit(declaration.origin.file);
    let Syntax::Item(index) = unit.configured.ast.node(declaration.origin.node).kind else {
        return Err(Declarations::error(
            report,
            declaration.origin,
            "source role lost its item",
        ));
    };
    Ok(unit.configured.ast.items[index as usize])
}

fn validate_marker(
    report: &Report,
    facts: &Declarations,
    declaration: Declaration,
) -> Result<(), Error> {
    if !report
        .resolution
        .source_bindings
        .as_deref()
        .is_some_and(|r| r.denies_implementation(declaration.definition))
    {
        return Err(Declarations::error(
            report,
            declaration.origin,
            "source compiler marker requires its declared implementation restriction",
        ));
    }
    let owner = facts
        .associated
        .owner(declaration.definition)
        .ok_or_else(|| {
            Declarations::error(report, declaration.origin, "missing source marker owner")
        })?;
    if facts
        .associated
        .owner_members(owner)
        .is_none_or(|m| !m.is_empty())
    {
        return Err(Declarations::error(
            report,
            declaration.origin,
            "source compiler marker must not declare members",
        ));
    }
    Ok(())
}
