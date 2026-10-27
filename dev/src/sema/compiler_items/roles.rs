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
use super::{Catalog, role_bit};
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

// Every role has an explicit stable index, checked against this cold mask width.
const _: [(); 1] = [(); (LangItem::ALL.len() <= u64::BITS as usize) as usize];
pub(in crate::sema) const fn bit(role: LangItem) -> u64 {
    1_u64 << role.index()
}

pub(super) struct Validated {
    pub(super) roles: u64,
    pub(super) methods: [Option<DefId>; 3],
    pub(super) pointee: Option<std::rc::Rc<super::pointee::Contracts>>,
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
            .is_none_or(|v| v.roles & (role_bit(role)) == 0)
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
            .is_none_or(|v| v.roles & (role_bit(role)) == 0)
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
    if report.package.staged_consumer().is_some() {
        // Full source validation belongs to the defining crate. Imported roles
        // retain their real identities and stay pending until selected validators
        // complete their existing per-role ledger; the first wire profile is
        // deliberately not applied wholesale to this retained full family.
        return facts
            .source_bindings
            .as_deref()
            .ok_or_else(|| Error::unlocated("staged roles lack their actual imported catalog"))?
            .require_provider_report(report)
            .map_err(Error::unlocated);
    }
    let Some(catalog) = facts.source_bindings.as_deref() else {
        return Ok(());
    };
    let mut complete = Validated {
        roles: 0,
        methods: [None; 3],
        pointee: None,
    };
    let mut callables = Vec::new();
    for role in [
        LangItem::Clone,
        LangItem::Copy,
        LangItem::LegacyReceiver,
        LangItem::TrivialClone,
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
            LangItem::TrivialClone => copying::validate_trivial(facts, catalog, &header)
                .map_err(|message| Declarations::error(report, declaration.origin, message))?,
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
                    complete.roles |= role_bit(LangItem::FnOnceOutput);
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
        complete.roles |= role_bit(role);
    }
    if let Some(declaration) = catalog.language(LangItem::DropGlue) {
        glue::validate(report, facts, catalog, declaration, complete.roles)?;
        complete.roles |= role_bit(LangItem::DropGlue);
    }
    validate_owned_members(report, catalog, complete.roles)?;
    if complete.roles == 0 {
        return Ok(());
    }
    require_distinct_pointee(report, facts, catalog, complete.roles)?;
    let output = catalog.language(LangItem::FnOnceOutput);
    publish(facts, complete, &callables, output)
}

/// Early metadata and later callable/marker completion share one owner, but
/// neither an overlapping family nor a bare bit mask is completion authority.
fn require_distinct_pointee(
    report: &Report,
    facts: &Declarations,
    catalog: &Catalog,
    roles: u64,
) -> Result<(), Error> {
    crate::sema::header_attributes::charge_source_contract_query(facts, 1)?;
    let Some(prior) = catalog.traits.as_deref() else {
        return Ok(());
    };
    let expected = (role_bit(LangItem::PointeeTrait))
        | (role_bit(LangItem::MetadataType))
        | if catalog.language(LangItem::DynMetadata).is_some() {
            role_bit(LangItem::DynMetadata)
        } else {
            0
        };
    if prior.roles != expected
        || prior.roles & roles != 0
        || prior.methods != [None; 3]
        || prior.pointee.is_none()
    {
        return Err(Error::unlocated("source roles were validated twice"));
    }
    super::pointee::authenticate(report, facts)
}

/// Publish only a fully checked role ledger and its actual callable members.
fn publish(
    facts: &mut Declarations,
    complete: Validated,
    callables: &[(Declaration, CallTrait, DefId)],
    output: Option<Declaration>,
) -> Result<(), Error> {
    let fresh = facts
        .source_bindings
        .as_deref()
        .is_some_and(|catalog| catalog.traits.is_none());
    crate::sema::header_attributes::reserve_source_contracts(
        facts,
        1,
        usize::from(fresh) * size_of::<Validated>(),
    )?;
    facts
        .associated
        .register_source_callables(callables, output)
        .map_err(|e| Error::at(crate::resolve::unit::FileId(0), 0, e))?;
    let catalog = facts.source_bindings.as_deref_mut().ok_or_else(|| {
        Error::at(
            crate::resolve::unit::FileId(0),
            0,
            "source catalog disappeared",
        )
    })?;
    if let Some(prior) = catalog.traits.as_deref_mut() {
        prior.roles |= complete.roles;
        prior.methods = complete.methods;
    } else {
        catalog.traits = Some(Box::new(complete));
    }
    Ok(())
}

fn validate_owned_members(report: &Report, catalog: &Catalog, roles: u64) -> Result<(), Error> {
    for role in [LangItem::CloneFn, LangItem::FnOnceOutput] {
        if let Some(declaration) = catalog.language(role)
            && roles & (role_bit(role)) == 0
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
            LangItem::FnOnce
                | LangItem::FnMut
                | LangItem::Fn
                | LangItem::Drop
                | LangItem::Destruct
                | LangItem::TrivialClone
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
                unsafe_: matches!(role, LangItem::Freeze | LangItem::TrivialClone),
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

/// Append only checked nominal declaration facts to the existing cold ledger.
/// This records no callable, intrinsic, constructor or field-access permission.
pub(super) fn complete_nominal(
    facts: &mut Declarations,
    role: LangItem,
    declaration: Declaration,
) -> Result<(), Error> {
    if !matches!(
        role,
        LangItem::PhantomData | LangItem::PanicLocation | LangItem::PanicInfo
    ) {
        return Err(Error::unlocated("invalid completed nominal role"));
    }
    let catalog = facts
        .source_bindings
        .as_deref()
        .ok_or_else(|| Error::unlocated("source nominal catalog disappeared"))?;
    if catalog.language(role) != Some(declaration) {
        return Err(Error::unlocated(
            "completed nominal role belongs to another declaration",
        ));
    }
    let fresh = catalog.traits.is_none();
    crate::sema::header_attributes::reserve_source_contracts(
        facts,
        1,
        usize::from(fresh) * size_of::<Validated>(),
    )?;
    let catalog = facts
        .source_bindings
        .as_deref_mut()
        .ok_or_else(|| Error::unlocated("source nominal catalog disappeared"))?;
    let ledger = catalog.traits.get_or_insert_with(|| {
        Box::new(Validated {
            roles: 0,
            methods: [None; 3],
            pointee: None,
        })
    });
    let bit = role_bit(role);
    if ledger.roles & bit != 0 {
        return Err(Error::unlocated("source nominal role was completed twice"));
    }
    ledger.roles |= bit;
    Ok(())
}

#[cfg(test)]
pub(super) const fn layout_bytes() -> usize {
    size_of::<Validated>()
}
