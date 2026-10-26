//! The first source packet admits semantic forms, never fixture names or counts.
use super::super::{
    Action, EnvironmentOrigin, FunctionBody, InputKind, Place, TraitFlags, ValueKind,
};
use super::super::{HeaderIndex, SourceRows};
use crate::support::crates::generics::Convention;
use crate::support::{
    bindings::LangItem, constness::Constness, crates::generics::Work, edition::Edition,
};

pub(super) fn validate(
    rows: &SourceRows,
    headers: &HeaderIndex<'_>,
    work: &mut Work,
) -> Result<(), &'static str> {
    validate_mode(rows, headers, work, NamespaceProfile::Root)
}
/// Private structural namespace inspection, never descriptor admission.
pub(super) fn validate_namespace(
    rows: &SourceRows,
    headers: &HeaderIndex<'_>,
    work: &mut Work,
) -> Result<(), &'static str> {
    validate_mode(rows, headers, work, NamespaceProfile::Inspection)
}
#[derive(Clone, Copy, PartialEq, Eq)]
enum NamespaceProfile {
    Root,
    Inspection,
    Contracts,
}
pub(super) fn validate_module_contracts(
    rows: &SourceRows,
    headers: &HeaderIndex<'_>,
    work: &mut Work,
) -> Result<(), &'static str> {
    validate_mode(rows, headers, work, NamespaceProfile::Contracts)
}
fn validate_mode(
    rows: &SourceRows,
    headers: &HeaderIndex<'_>,
    work: &mut Work,
    profile: NamespaceProfile,
) -> Result<(), &'static str> {
    work.charge(1)?;
    if !rows.body_units.is_empty() {
        return Err("source numbering units require their wider explicit profile");
    }
    if rows.policy.edition != Edition::Rust2024 || rows.policy.provider.0 != 0 {
        return Err("ordinary source functions require their own Rust2024 provider profile");
    }
    let sizing = bindings(rows, headers, work, false)?;
    recipes(rows, work)?;
    for header in &rows.traits {
        work.charge(1)?;
        let flags = header.flags.bits();
        if flags & TraitFlags::RESTRICTED_IMPL != 0 && profile != NamespaceProfile::Contracts {
            return Err(
                "restricted trait requires version-two complete implementation-domain columns",
            );
        }
        if flags & (TraitFlags::CONST | TraitFlags::AUTO | TraitFlags::UNSAFE) != 0 {
            return Err("source const/auto/unsafe traits require a wider explicit profile");
        }
        if flags
            & (TraitFlags::FUNDAMENTAL
                | TraitFlags::SPECIALIZATION
                | TraitFlags::COINDUCTIVE
                | TraitFlags::DENY_IMPL)
            != 0
            && (!sizing.contains(&Some(header.declaration)) || flags & TraitFlags::DENY_IMPL == 0)
        {
            return Err(
                "source sizing attributes require exact sizing role and deny-impl ownership",
            );
        }
        if flags & TraitFlags::DYN_INCOMPATIBLE != 0 && Some(header.declaration) != sizing[0] {
            return Err(
                "source dyn incompatibility claim requires its actual supported Sized role",
            );
        }
    }
    functions(rows, headers, work, profile != NamespaceProfile::Root)?;
    for header in &rows.implementations {
        work.charge(1)?;
        if header.constant || header.negative || header.unsafe_ {
            return Err(
                "source const/negative/unsafe implementation requires a wider explicit profile",
            );
        }
    }
    for clause in &rows.clauses {
        work.charge(1)?;
        ordinary(clause.requirement)?;
    }
    originals(rows, work)
}
fn functions(
    rows: &SourceRows,
    headers: &HeaderIndex<'_>,
    work: &mut Work,
    modules: bool,
) -> Result<(), &'static str> {
    for declaration in &rows.declarations {
        work.charge(1)?;
        if !modules
            && declaration.definition.kind == super::super::DefinitionKind::Module
            && declaration.definition.parent.is_some()
        {
            return Err("nested source modules require a wider source namespace profile");
        }
    }
    for header in &rows.functions {
        work.charge(1)?;
        if header.constness != Constness::Ordinary
            || header.original.unsafe_
            || header.original.variadic
            || header.original.convention != Convention::Rust
            || header.callable.unsafe_
            || header.callable.variadic
            || header.callable.convention != Convention::Rust
        {
            return Err(
                "source nonordinary callable effect/safety/convention requires a wider profile",
            );
        }
        if matches!(header.body, FunctionBody::Original(_)) && header.parent.is_some() {
            return Err("source provided associated bodies require a wider explicit profile");
        }
        if matches!(header.body, FunctionBody::Original(_)) && header.regions.total != 0 {
            return Err("source original region formals require their complete wider body profile");
        }
        let parent = rows.declarations[header.declaration.0 as usize]
            .definition
            .parent
            .ok_or("source function lacks its actual lexical declaration owner")?;
        let parent_kind = rows.declarations[parent.0 as usize].definition.kind;
        match header.body {
            FunctionBody::Absent
                if header.parent != Some(parent)
                    || parent_kind != super::super::DefinitionKind::Trait =>
            {
                return Err(
                    "source body absence is supported only for actual required trait members",
                );
            }
            FunctionBody::Original(_) if parent_kind != super::super::DefinitionKind::Module => {
                return Err("source original requires its actual supported root module owner");
            }
            _ => {}
        }
        headers.header(header.declaration, work)?;
    }
    Ok(())
}
fn originals(rows: &SourceRows, work: &mut Work) -> Result<(), &'static str> {
    for original in &rows.originals {
        work.charge(1)?;
        if original.context != super::super::CallContext::Runtime
            || !original.variables.is_empty()
            || !original.projections.is_empty()
            || !original.calls.is_empty()
            || !original.arguments.is_empty()
            || !original.openings.is_empty()
        {
            return Err("source contextual/borrow/call original requires a wider explicit profile");
        }
        for value in &original.values {
            work.charge(1)?;
            if !matches!(
                value.kind,
                ValueKind::Unit | ValueKind::Read(Place::Local(_)) | ValueKind::Constrain(_)
            ) {
                return Err("source value operation requires a wider explicit profile");
            }
        }
        for statement in &original.statements {
            work.charge(1)?;
            if matches!(
                statement.action,
                Action::Project(_)
                    | Action::Store {
                        place: Place::Projection(_),
                        ..
                    }
                    | Action::CheckInitialized(Place::Projection(_))
                    | Action::Mention(Place::Projection(_))
            ) {
                return Err("source projected operation requires a wider explicit profile");
            }
        }
    }
    for input in &rows.body_inputs {
        work.charge(1)?;
        if matches!(
            input.environment,
            EnvironmentOrigin::ConstTraitDefault { .. }
        ) {
            return Err("source defining default requires its wider imported preparation profile");
        }
        if let Some(builtins) = &input.builtins {
            if matches!(
                builtins.environment,
                EnvironmentOrigin::ConstTraitDefault { .. }
            ) {
                return Err("source defining builtin environment requires a wider source profile");
            }
            for request in &builtins.predicates {
                work.charge(1)?;
                ordinary(request.value)?;
            }
        }
        for request in &input.type_assumptions {
            work.charge(1)?;
            ordinary(request.value)?;
        }
        for normalization in &input.normalizations {
            work.charge(1)?;
            if !normalization.owned.is_empty() {
                return Err("source owned normalization requires a wider explicit profile");
            }
        }
        for obligation in &input.obligations {
            work.charge(1)?;
            if matches!(
                obligation.kind,
                InputKind::Variable { .. }
                    | InputKind::Membership { .. }
                    | InputKind::LocalBorrow(_)
                    | InputKind::ScopedWellFormed { .. }
            ) {
                return Err("source allocation/scoped input requires a wider explicit profile");
            }
        }
    }
    Ok(())
}
fn recipes(rows: &SourceRows, work: &mut Work) -> Result<(), &'static str> {
    work.charge(rows.function_applications.len())?;
    if !rows.function_applications.is_empty() {
        return Err("source application recipes require a wider source profile");
    }
    for row in &rows.binders {
        work.charge(1)?;
        if matches!(row.kind, super::super::BinderKind::CallSignature { .. }) {
            return Err("source call binder requires a wider source profile");
        }
    }
    for row in &rows.types {
        work.charge(1)?;
        if matches!(
            row,
            super::super::Type::FunctionItem(_) | super::super::Type::ApplicationSignature(_)
        ) {
            return Err("source application recipe requires a wider source profile");
        }
    }
    for row in &rows.regions {
        work.charge(1)?;
        if matches!(row, super::super::Region::Variable { .. }) {
            return Err("source body allocation recipe requires a wider source profile");
        }
    }
    Ok(())
}
fn ordinary(request: super::super::Requirement) -> Result<(), &'static str> {
    if matches!(request, super::super::Requirement::Outlives(_)) {
        return Err("standalone regional clauses require their explicit wider header consumer");
    }
    if matches!(request, super::super::Requirement::Trait { effect, .. } if effect != Constness::Ordinary)
    {
        return Err("source qualified body request requires a wider source profile");
    }
    Ok(())
}
pub(super) fn bindings(
    rows: &SourceRows,
    headers: &HeaderIndex<'_>,
    work: &mut Work,
    provided: bool,
) -> Result<[Option<crate::support::crates::owner_catalog::DeclarationRef>; 3], &'static str> {
    work.charge(1)?;
    if rows.policy.roles.start != 0 || rows.policy.roles.end as usize != rows.roles.len() {
        return Err("source role policy omits part of its exact binding inventory");
    }
    let mut declarations = [None; LangItem::ALL.len()];
    for role in &rows.roles {
        work.charge(declarations.len())?;
        let allowed = matches!(
            role.role,
            LangItem::Sized
                | LangItem::MetaSized
                | LangItem::PointeeSized
                | LangItem::LegacyReceiver
                | LangItem::Clone
                | LangItem::CloneFn
                | LangItem::Copy
        ) || provided && matches!(role.role, LangItem::Destruct | LangItem::Freeze);
        if !allowed {
            return Err("source role requires a wider explicit source profile");
        }
        let address = role.declaration;
        if address.artifact != super::super::ArtifactRef(0)
            || declarations.contains(&Some(address.declaration))
            || declarations[role.role.index()].is_some()
        {
            return Err("source role binding is duplicated or belongs to a foreign provider");
        }
        match (role.role, headers.header(address.declaration, work)?) {
            (LangItem::CloneFn, Some(super::super::HeaderRef::Function(_))) => {}
            (LangItem::CloneFn, _)
            | (
                _,
                None
                | Some(
                    super::super::HeaderRef::Function(_)
                    | super::super::HeaderRef::Implementation(_),
                ),
            ) => return Err("source role has a different actual declaration/header kind"),
            (_, Some(super::super::HeaderRef::Trait(_))) => {}
        }
        declarations[role.role.index()] = Some(address.declaration);
    }
    if let Some(clone) = declarations[LangItem::CloneFn.index()] {
        let Some(super::super::HeaderRef::Function(index)) = headers.header(clone, work)? else {
            return Err("source clone_fn has no exact method header");
        };
        if rows.functions[index as usize].parent != declarations[LangItem::Clone.index()]
            || declarations[LangItem::Clone.index()].is_none()
        {
            return Err("source clone_fn belongs to another actual Clone declaration");
        }
    }
    let sizing = [
        declarations[LangItem::Sized.index()],
        declarations[LangItem::MetaSized.index()],
        declarations[LangItem::PointeeSized.index()],
    ];
    if sizing.contains(&None) {
        return Err("source functions require the real complete sizing trio");
    }
    for slot in &rows.slots {
        work.charge(1)?;
        if let super::super::ParameterKind::Type {
            sized: Some(address),
        } = slot.kind
            && (address.artifact != super::super::ArtifactRef(0)
                || Some(address.declaration) != sizing[0])
        {
            return Err("source sized formal names a foreign or different actual Sized role");
        }
    }
    Ok(sizing)
}
