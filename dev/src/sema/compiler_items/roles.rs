//! Source roles become usable only after their actual complete headers validate.
mod copying;
mod destructibility;
mod freezing;
mod glue;
pub(crate) use destructibility::destruct_predicate;
pub(crate) use freezing::predicate as freeze_predicate;
pub(crate) use glue::member as drop_glue_member;
pub(super) mod admitted_headers;
mod invocation;
pub(crate) use invocation::definition as callable_role;
mod unsizing;
use super::Catalog;
use crate::{
    resolve::{
        bindings::Declaration,
        expansion::Report,
        model::{DefId, DefKind},
        package::Error,
    },
    sema::model::{Declarations, Function},
    support::{bindings::LangItem, call_traits::CallTrait},
    syntax::{ast::Kind as Syntax, item::ItemKind},
    types::{Kind, TypeId, parameters::predicates::Bound},
};
pub(crate) use unsizing::{kind as unsizing_role, predicate as unsizing_predicate};

pub(super) struct Validated {
    roles: u32,
    pub(super) methods: [Option<DefId>; 3],
}
impl Catalog {
    /// The declaration-validation ledger is complete and its retained catalog
    /// identity is authentic. This grants no impl/instance/body proof; imported
    /// consumers must still authenticate their actual canonical headers.
    pub(in crate::sema) fn validated_role(
        &self,
        role: LangItem,
    ) -> Result<Option<super::Role>, &'static str> {
        let Some(declaration) = self.role(role)? else {
            return Ok(None);
        };
        if let Some(index) = match role {
            LangItem::Sized => Some(0),
            LangItem::MetaSized => Some(1),
            LangItem::PointeeSized => Some(2),
            _ => None,
        } {
            if self.sizing.as_ref().map(|ids| ids[index]) != Some(declaration.definition) {
                return Err("source sizing role requires completed declaration validation");
            }
        } else if self
            .traits
            .as_ref()
            .is_none_or(|v| v.roles & (1 << role.index()) == 0)
        {
            return Err("source trait role requires completed declaration validation");
        }
        Ok(Some(declaration))
    }

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
    if report.package.source_client().is_some() {
        return admitted_headers::traits(report, facts);
    }
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
        LangItem::Freeze,
        LangItem::Unsize,
        LangItem::CoerceUnsized,
    ] {
        let Some(declaration) = catalog.language(role) else {
            continue;
        };
        let header = header(report, facts, declaration, role)?;
        let self_type = header.self_type;
        match role {
            LangItem::Clone | LangItem::Copy | LangItem::LegacyReceiver => copying::validate(
                report,
                facts,
                catalog,
                declaration,
                role,
                &header,
                &mut complete.roles,
            )?,
            LangItem::Tuple => validate_marker(report, facts, declaration)?,
            LangItem::Freeze => freezing::validate(report, facts, declaration)?,
            LangItem::Unsize | LangItem::CoerceUnsized => {
                unsizing::validate(report, facts, catalog, declaration, role, self_type)?;
            }
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
    if let Some(declaration) = catalog.language(LangItem::DropGlue) {
        glue::validate(report, facts, catalog, declaration, complete.roles)?;
        complete.roles |= 1 << LangItem::DropGlue.index();
    }
    validate_owned_members(report, catalog, complete.roles)?;
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

fn validate_owned_members(report: &Report, catalog: &Catalog, roles: u32) -> Result<(), Error> {
    for role in [LangItem::CloneFn, LangItem::FnOnceOutput] {
        if let Some(declaration) = catalog.language(role)
            && roles & (1 << role.index()) == 0
        {
            return Err(Declarations::error(
                report,
                declaration.origin,
                "source language member has no validated actual trait owner",
            ));
        }
    }
    Ok(())
}

fn header<'a>(
    report: &Report,
    facts: &'a Declarations,
    declaration: Declaration,
    role: LangItem,
) -> Result<super::headers::Trait<'a>, Error> {
    let error = |message| Declarations::error(report, declaration.origin, message);
    let header = super::headers::Trait::source(report, facts, declaration).map_err(error)?;
    // Clone may be an ordinary or const source trait. Preserve that actual
    // declaration fact; role admission supplies no executable const capability.
    let constant = if role == LangItem::Clone {
        header.flags.constant
    } else {
        matches!(
            role,
            LangItem::FnOnce | LangItem::FnMut | LangItem::Fn | LangItem::Drop | LangItem::Destruct
        )
    };
    let expected = usize::from(matches!(
        role,
        LangItem::FnOnce
            | LangItem::FnMut
            | LangItem::Fn
            | LangItem::Unsize
            | LangItem::CoerceUnsized
    ));
    header
        .profile(
            &facts.types,
            super::headers::Flags {
                unsafe_: role == LangItem::Freeze,
                constant,
                auto_: role == LangItem::Freeze,
            },
            expected,
        )
        .map_err(error)?;
    Ok(header)
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
