//! The first source packet admits semantic forms, never fixture names or counts.
use super::super::{
    Action, EnvironmentOrigin, FunctionBody, InputKind, Place, TraitFlags, ValueKind,
};
use super::super::{HeaderIndex, SourceRows};
#[path = "free_protocols.rs"]
mod callable_traits;
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
    FeaturePredicates,
    Definitions,
    Callables,
}
pub(super) fn validate_module_contracts(
    rows: &SourceRows,
    headers: &HeaderIndex<'_>,
    work: &mut Work,
) -> Result<(), &'static str> {
    validate_mode(rows, headers, work, NamespaceProfile::Contracts)
}
pub(super) fn validate_module_feature_predicates(
    rows: &SourceRows,
    headers: &HeaderIndex<'_>,
    work: &mut Work,
) -> Result<(), &'static str> {
    validate_mode(rows, headers, work, NamespaceProfile::FeaturePredicates)
}
pub(super) fn validate_module_definitions(
    rows: &SourceRows,
    headers: &HeaderIndex<'_>,
    work: &mut Work,
) -> Result<(), &'static str> {
    validate_mode(rows, headers, work, NamespaceProfile::Definitions)
}
pub(super) fn validate_module_callables(
    rows: &SourceRows,
    headers: &HeaderIndex<'_>,
    work: &mut Work,
) -> Result<(), &'static str> {
    validate_mode(rows, headers, work, NamespaceProfile::Callables)
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
    let roles = bindings(
        rows,
        headers,
        work,
        if profile == NamespaceProfile::Callables {
            BindingProfile::Callables
        } else {
            BindingProfile::Ordinary
        },
    )?;
    let sizing = roles.sizing;
    let features = matches!(
        profile,
        NamespaceProfile::FeaturePredicates
            | NamespaceProfile::Definitions
            | NamespaceProfile::Callables
    );
    recipes(rows, work, features)?;
    for header in &rows.traits {
        work.charge(1)?;
        let flags = header.flags.bits();
        if roles.destruct == Some(header.declaration) {
            callable_traits::destruct(
                rows,
                headers,
                header,
                sizing[2].ok_or("source Destruct lacks its actual PointeeSized role")?,
                work,
            )?;
            continue;
        }
        if roles.freeze == Some(header.declaration) {
            callable_traits::freeze(rows, headers, header, work)?;
            continue;
        }
        if flags & TraitFlags::RESTRICTED_IMPL != 0
            && !matches!(
                profile,
                NamespaceProfile::Contracts
                    | NamespaceProfile::FeaturePredicates
                    | NamespaceProfile::Definitions
                    | NamespaceProfile::Callables
            )
        {
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
    functions(rows, headers, work, profile)?;
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
        ordinary(&clause.requirement)?;
    }
    originals(
        rows,
        headers,
        work,
        features,
        profile == NamespaceProfile::Callables,
    )
}
fn functions(
    rows: &SourceRows,
    headers: &HeaderIndex<'_>,
    work: &mut Work,
    profile: NamespaceProfile,
) -> Result<(), &'static str> {
    for declaration in &rows.declarations {
        work.charge(1)?;
        if profile == NamespaceProfile::Root
            && declaration.definition.kind == super::super::DefinitionKind::Module
            && declaration.definition.parent.is_some()
        {
            return Err("nested source modules require a wider source namespace profile");
        }
    }
    for header in &rows.functions {
        work.charge(1 + 2 * rows.exact_callables.len().checked_ilog2().unwrap_or(0) as usize)?;
        let free_effects = profile == NamespaceProfile::Callables && header.parent.is_none();
        if (!free_effects && header.constness != Constness::Ordinary)
            || header.constness == Constness::Conditional
            || (!free_effects && header.original.unsafe_)
            || header.original.variadic
            || !admitted_convention(
                rows,
                header.declaration,
                header.original.convention,
                profile,
            )
            || (!free_effects && header.callable.unsafe_)
            || header.callable.variadic
            || !admitted_convention(
                rows,
                header.declaration,
                header.callable.convention,
                profile,
            )
        {
            return Err(
                "source nonordinary callable effect/safety/convention requires a wider profile",
            );
        }
        if free_effects
            && (header.original.unsafe_ != header.callable.unsafe_
                || header.original.convention != header.callable.convention)
        {
            return Err(
                "source free callable changed its actual original/public safety or convention",
            );
        }
        let primitive_method = matches!(
            profile,
            NamespaceProfile::FeaturePredicates
                | NamespaceProfile::Definitions
                | NamespaceProfile::Callables
        ) && primitive_method(rows, headers, header, work)?;
        if matches!(header.body, FunctionBody::Original(_))
            && header.parent.is_some()
            && !primitive_method
        {
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
            FunctionBody::Original(_)
                if parent_kind != super::super::DefinitionKind::Module && !primitive_method =>
            {
                return Err("source original requires its actual supported root module owner");
            }
            _ => {}
        }
        headers.header(header.declaration, work)?;
    }
    Ok(())
}
fn primitive_method(
    rows: &SourceRows,
    headers: &HeaderIndex<'_>,
    function: &super::super::FunctionHeader,
    work: &mut Work,
) -> Result<bool, &'static str> {
    let Some(parent) = function.parent else {
        return Ok(false);
    };
    let Some(super::super::HeaderRef::Implementation(index)) = headers.header(parent, work)? else {
        return Ok(false);
    };
    let implementation = &rows.implementations[index as usize];
    work.charge(4)?;
    if implementation.self_definition != parent
        || implementation.constant
        || implementation.negative
        || implementation.unsafe_
        || implementation.region_captures != 0
        || implementation.region_bounds.start != implementation.region_bounds.end
        || !matches!(
            rows.types.get(implementation.self_type.0 as usize),
            Some(super::super::Type::Scalar(_))
        )
        || function.receiver.is_some()
        || function.regions.total != 0
    {
        return Ok(false);
    }
    for owner in [implementation.scope, function.scope] {
        if headers.scopes().complete_slots(owner, work)? != 0 {
            return Ok(false);
        }
    }
    for signature in [function.original, function.callable] {
        work.charge(1)?;
        if rows
            .binders
            .get(signature.binder.0 as usize)
            .is_none_or(|binder| binder.count != 0)
            || !matches!(
                rows.types.get(signature.result.0 as usize),
                Some(super::super::Type::Scalar(_))
            )
        {
            return Ok(false);
        }
        for parameter in
            &rows.roots[signature.parameters.start as usize..signature.parameters.end as usize]
        {
            work.charge(1)?;
            if !matches!(
                rows.types.get(parameter.0 as usize),
                Some(super::super::Type::Scalar(_))
            ) {
                return Ok(false);
            }
        }
    }
    Ok(true)
}
fn originals(
    rows: &SourceRows,
    headers: &HeaderIndex<'_>,
    work: &mut Work,
    features: bool,
    callables: bool,
) -> Result<(), &'static str> {
    for original in &rows.originals {
        work.charge(1)?;
        let expected = if callables {
            callable_traits::original_context(rows, headers, original, work)?
        } else {
            super::super::CallContext::Runtime
        };
        if original.context != expected
            || !original.variables.is_empty()
            || !original.projections.is_empty()
            || !features && (!original.calls.is_empty() || !original.arguments.is_empty())
            || !original.openings.is_empty()
        {
            return Err("source contextual/borrow/call original requires a wider explicit profile");
        }
        for value in &original.values {
            work.charge(1)?;
            if !(matches!(
                value.kind,
                ValueKind::Unit | ValueKind::Read(Place::Local(_)) | ValueKind::Constrain(_)
            ) || features
                && matches!(
                    value.kind,
                    ValueKind::Function(_) | ValueKind::Call { .. } | ValueKind::Scalar(_)
                ))
            {
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
                ordinary(&request.value)?;
            }
        }
        for request in &input.type_assumptions {
            work.charge(1)?;
            ordinary(&request.value)?;
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
fn recipes(rows: &SourceRows, work: &mut Work, features: bool) -> Result<(), &'static str> {
    work.charge(rows.function_applications.len())?;
    if !features && !rows.function_applications.is_empty() {
        return Err("source application recipes require a wider source profile");
    }
    for row in &rows.binders {
        work.charge(1)?;
        if matches!(row.kind, super::super::BinderKind::CallSignature { .. })
            && (!features || row.count != 0)
        {
            return Err("source call binder requires a wider source profile");
        }
    }
    for row in &rows.types {
        work.charge(1)?;
        if !features
            && matches!(
                row,
                super::super::Type::FunctionItem(_) | super::super::Type::ApplicationSignature(_)
            )
        {
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
fn ordinary(request: &super::super::Requirement) -> Result<(), &'static str> {
    if matches!(request, super::super::Requirement::Outlives(_)) {
        return Err("standalone regional clauses require their explicit wider header consumer");
    }
    if matches!(request, super::super::Requirement::Trait { effect, .. } if *effect != Constness::Ordinary)
    {
        return Err("source qualified body request requires a wider source profile");
    }
    Ok(())
}
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum BindingProfile {
    Ordinary,
    Provided,
    Callables,
}
pub(super) struct Bindings {
    sizing: [Option<crate::support::crates::owner_catalog::DeclarationRef>; 3],
    destruct: Option<crate::support::crates::owner_catalog::DeclarationRef>,
    freeze: Option<crate::support::crates::owner_catalog::DeclarationRef>,
}
pub(super) fn bindings(
    rows: &SourceRows,
    headers: &HeaderIndex<'_>,
    work: &mut Work,
    profile: BindingProfile,
) -> Result<Bindings, &'static str> {
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
        ) || (matches!(
            profile,
            BindingProfile::Provided | BindingProfile::Callables
        ) && matches!(role.role, LangItem::Destruct | LangItem::Freeze));
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
    Ok(Bindings {
        sizing,
        destruct: if profile == BindingProfile::Callables {
            declarations[LangItem::Destruct.index()]
        } else {
            None
        },
        freeze: if profile == BindingProfile::Callables {
            declarations[LangItem::Freeze.index()]
        } else {
            None
        },
    })
}

fn admitted_convention(
    rows: &SourceRows,
    declaration: crate::support::crates::owner_catalog::DeclarationRef,
    convention: Convention,
    profile: NamespaceProfile,
) -> bool {
    convention == Convention::Rust
        || (matches!(
            profile,
            NamespaceProfile::Definitions | NamespaceProfile::Callables
        ) && convention == Convention::C
            && rows
                .exact_callables
                .binary_search_by_key(&declaration.0, |row| row.declaration.0)
                .is_ok())
}
