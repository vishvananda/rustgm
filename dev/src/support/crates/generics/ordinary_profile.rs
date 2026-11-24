//! The first source packet admits semantic forms, never fixture names or counts.
use super::super::{
    Action, EnvironmentOrigin, FunctionBody, InputKind, Place, Profile, TraitFlags, ValueKind,
};
use super::super::{HeaderIndex, SourceRows, interval_roles};
#[path = "rust_call_roles.rs"]
mod callable_roles;
#[path = "free_protocols.rs"]
mod callable_traits;
#[path = "frame_captures.rs"]
mod captured;
#[path = "conversion_profile.rs"]
mod conversions;
use crate::support::crates::generics::Convention;
use crate::support::{
    bindings::LangItem, constness::Constness, crates::generics::Work, edition::Edition,
    polarity::ImplPolarity,
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
    Safety,
    Conversions,
    CompilerOperations,
}
impl NamespaceProfile {
    fn conversions(self) -> bool {
        matches!(self, Self::Conversions | Self::CompilerOperations)
    }
    fn callables(self) -> bool {
        matches!(
            self,
            Self::Callables | Self::Safety | Self::Conversions | Self::CompilerOperations
        )
    }
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
pub(super) fn validate_module_safety(
    rows: &SourceRows,
    headers: &HeaderIndex<'_>,
    work: &mut Work,
) -> Result<(), &'static str> {
    validate_mode(rows, headers, work, NamespaceProfile::Safety)
}
pub(super) fn validate_module_conversions(
    rows: &SourceRows,
    headers: &HeaderIndex<'_>,
    work: &mut Work,
) -> Result<(), &'static str> {
    validate_mode(rows, headers, work, NamespaceProfile::Conversions)
}
pub(super) fn validate_compiler_operations(
    rows: &SourceRows,
    headers: &HeaderIndex<'_>,
    work: &mut Work,
) -> Result<(), &'static str> {
    validate_mode(rows, headers, work, NamespaceProfile::CompilerOperations)
}
/// Version thirteen is version ten's namespace plus general language roles.
pub(super) fn validate_module_bodies(
    rows: &SourceRows,
    headers: &HeaderIndex<'_>,
    work: &mut Work,
) -> Result<(), &'static str> {
    validate_with(
        rows,
        headers,
        work,
        NamespaceProfile::Conversions,
        BindingProfile::Languages,
    )
}
pub(super) fn validate_body_operations(
    rows: &SourceRows,
    headers: &HeaderIndex<'_>,
    work: &mut Work,
) -> Result<(), &'static str> {
    validate_with(
        rows,
        headers,
        work,
        NamespaceProfile::CompilerOperations,
        BindingProfile::BodyOperations,
    )
}
fn validate_mode(
    rows: &SourceRows,
    headers: &HeaderIndex<'_>,
    work: &mut Work,
    profile: NamespaceProfile,
) -> Result<(), &'static str> {
    let roles = if profile.conversions() {
        if profile == NamespaceProfile::CompilerOperations {
            BindingProfile::CompilerOperations
        } else {
            BindingProfile::Conversions
        }
    } else if profile.callables() {
        BindingProfile::Callables
    } else {
        BindingProfile::Ordinary
    };
    validate_with(rows, headers, work, profile, roles)
}
fn validate_with(
    rows: &SourceRows,
    headers: &HeaderIndex<'_>,
    work: &mut Work,
    profile: NamespaceProfile,
    binding: BindingProfile,
) -> Result<(), &'static str> {
    work.charge(1)?;
    if !profile.conversions() && !rows.body_units.is_empty() {
        return Err("source numbering units require their wider explicit profile");
    }
    if rows.policy.edition != Edition::Rust2024 || rows.policy.provider.0 != 0 {
        return Err("ordinary source functions require their own Rust2024 provider profile");
    }
    let roles = bindings(rows, headers, work, binding)?;
    let features = matches!(
        profile,
        NamespaceProfile::FeaturePredicates
            | NamespaceProfile::Definitions
            | NamespaceProfile::Callables
            | NamespaceProfile::Safety
            | NamespaceProfile::Conversions
            | NamespaceProfile::CompilerOperations
    );
    recipes(rows, work, features, profile.conversions())?;
    trait_headers(rows, headers, work, profile, &roles)?;
    functions(rows, headers, work, profile, &roles, binding.languages())?;
    for header in &rows.implementations {
        work.charge(1)?;
        // Version thirteen admits every trait impl's own safety and polarity;
        // the ledger already proved its trait is this packet's own, and each
        // consumer re-proves both against the trait.
        let reservation = header.polarity == ImplPolarity::Reservation;
        if reservation && binding != BindingProfile::BodyOperations {
            return Err("reservation implementation transport requires a wider metadata profile");
        }
        if header.constant
            && !reservation
            && !roles.const_binary_impl(rows, headers, header, work)?
            || (!header.polarity.is_positive() || header.unsafe_)
                && !(roles.languages && header.trait_application.is_some())
        {
            return Err(
                "source const/negative/unsafe implementation requires a wider explicit profile",
            );
        }
        if roles.languages {
            member_safety(rows, headers, header, work)?;
        }
    }
    for clause in &rows.clauses {
        work.charge(1)?;
        ordinary(&clause.requirement, roles.callables.callable(clause.owner))?;
    }
    originals(rows, headers, work, profile, binding, features)?;
    body_inputs(rows, work, profile.conversions(), binding.languages())
}
/// Version thirteen: an implementation member's public safety is its trait
/// member's, as rustc's E0053 requires of the impl that declares it.
fn member_safety(
    rows: &SourceRows,
    headers: &HeaderIndex<'_>,
    implementation: &super::super::ImplHeader,
    work: &mut Work,
) -> Result<(), &'static str> {
    for link in implementation.members.slice(&rows.member_links)? {
        work.charge(1)?;
        let (Some(member), Some(actual)) = (link.trait_member, link.implementation) else {
            continue;
        };
        let (
            Some(super::super::HeaderRef::Function(expected)),
            Some(super::super::HeaderRef::Function(actual)),
        ) = (
            headers.header(member.declaration, work)?,
            headers.header(actual, work)?,
        )
        else {
            continue;
        };
        if rows.functions[expected as usize].callable.unsafe_
            != rows.functions[actual as usize].callable.unsafe_
        {
            return Err("source implementation member changed its trait member's safety");
        }
    }
    Ok(())
}
fn trait_headers(
    rows: &SourceRows,
    headers: &HeaderIndex<'_>,
    work: &mut Work,
    profile: NamespaceProfile,
    roles: &Bindings,
) -> Result<(), &'static str> {
    let sizing = roles.sizing;
    for header in &rows.traits {
        work.charge(1)?;
        let flags = header.flags.bits();
        if flags & TraitFlags::INCOHERENT_INHERENT & !roles.definition_policies != 0 {
            return Err("source incoherent trait requires the declaration policy profile");
        }
        if flags & TraitFlags::DISPATCH_POLICIES & !roles.definition_policies != 0 {
            return Err("source trait dispatch requires the declaration policy profile");
        }
        let role_header = super::super::TraitHeader {
            flags: TraitFlags::from_bits(flags & !roles.definition_policies)?,
            ..*header
        };
        if roles.destruct == Some(header.declaration) {
            // Destruct requires its actual forced-dyn restriction.
            let destruct = super::super::TraitHeader {
                flags: TraitFlags::from_bits(
                    flags & !(roles.definition_policies & !TraitFlags::DYN_INCOMPATIBLE),
                )?,
                ..*header
            };
            callable_traits::destruct(
                rows,
                headers,
                &destruct,
                sizing[2].ok_or("source Destruct lacks its actual PointeeSized role")?,
                work,
            )?;
            continue;
        }
        if roles.no_drop == Some(header.declaration) {
            require_no_drop_header(header)?;
            continue;
        }
        if roles.freeze == Some(header.declaration) {
            callable_traits::freeze(rows, headers, &role_header, work)?;
            continue;
        }
        // Version thirteen: an auto trait is admitted only as its exact empty
        // declaration; a role-bound marker also keeps its local role's flags.
        if roles.languages && flags & TraitFlags::AUTO != 0 {
            callable_traits::automatic(
                rows,
                headers,
                &role_header,
                roles.marker(header.declaration),
                work,
            )?;
            continue;
        }
        if callable_roles::trait_header(rows, headers, &role_header, roles.callables, work)?
            .is_some()
        {
            continue;
        }
        if flags & TraitFlags::RESTRICTED_IMPL != 0
            && !matches!(
                profile,
                NamespaceProfile::Contracts
                    | NamespaceProfile::FeaturePredicates
                    | NamespaceProfile::Definitions
                    | NamespaceProfile::Callables
                    | NamespaceProfile::Safety
                    | NamespaceProfile::Conversions
                    | NamespaceProfile::CompilerOperations
            )
        {
            return Err(
                "restricted trait requires version-two complete implementation-domain columns",
            );
        }
        let conversion = profile.conversions() && roles.conversion == Some(header.declaration);
        let binary = roles.binary_operators.contains(&Some(header.declaration));
        let range = roles
            .range
            .is_some_and(|family| family.owner == header.declaration);
        if conversion {
            conversions::trait_header(rows, headers, header, work)?;
        }
        // Version thirteen also admits an unsafe trait that is not auto.
        if flags & TraitFlags::AUTO != 0
            || flags & TraitFlags::UNSAFE != 0 && !roles.languages
            || flags & TraitFlags::CONST != 0 && !conversion && !binary && !range
        {
            return Err("source const/auto/unsafe traits require a wider explicit profile");
        }
        ordinary_sizing_policy(header.declaration, flags, role_header.flags, sizing)?;
    }
    Ok(())
}
fn require_no_drop_header(header: &super::super::TraitHeader) -> Result<(), &'static str> {
    if header.flags.bits() != (TraitFlags::DENY_IMPL | TraitFlags::DYN_INCOMPATIBLE)
        || header.written_parameters != 0
        || header.region_captures != 0
        || header.members.start != header.members.end
        || header.written_supertraits.start != header.written_supertraits.end
    {
        return Err("source NoDrop requires its exact empty deny-impl declaration");
    }
    Ok(())
}

/// An associated original is admitted only by the method family its profile
/// owns: primitive methods, conversion methods and, under version thirteen
/// (`bodies`), inherent methods whose impl captures lifetimes.
///
/// A conversion method may belong to an unsafe impl exactly when
/// `roles.languages` holds, the one fact by which the impl loop admits an
/// impl's unsafe byte; `bodies` is not that gate. Primitive methods keep
/// refusing unsafe impls; under version thirteen every primitive method is
/// also a conversion method, so that refusal is not final.
fn ordinary_sizing_policy(
    declaration: crate::support::crates::owner_catalog::DeclarationRef,
    flags: u16,
    role_flags: TraitFlags,
    sizing: [Option<crate::support::crates::owner_catalog::DeclarationRef>; 3],
) -> Result<(), &'static str> {
    let sizing_role = sizing.contains(&Some(declaration));
    let sizing_only = TraitFlags::FUNDAMENTAL
        | TraitFlags::SPECIALIZATION
        | TraitFlags::COINDUCTIVE
        | TraitFlags::DENY_IMPL;
    let sizing_flags = if sizing_role {
        flags
    } else {
        role_flags.bits()
    };
    if sizing_flags & sizing_only != 0 && (!sizing_role || flags & TraitFlags::DENY_IMPL == 0) {
        return Err("source sizing attributes require exact sizing role and deny-impl ownership");
    }
    if role_flags.bits() & TraitFlags::DYN_INCOMPATIBLE != 0 && Some(declaration) != sizing[0] {
        return Err("source dyn incompatibility claim requires its actual supported Sized role");
    }
    Ok(())
}

fn associated_method(
    rows: &SourceRows,
    headers: &HeaderIndex<'_>,
    header: &super::super::FunctionHeader,
    work: &mut Work,
    profile: NamespaceProfile,
    roles: &Bindings,
    bodies: bool,
) -> Result<bool, &'static str> {
    let primitive = matches!(
        profile,
        NamespaceProfile::FeaturePredicates
            | NamespaceProfile::Definitions
            | NamespaceProfile::Callables
            | NamespaceProfile::Safety
            | NamespaceProfile::Conversions
            | NamespaceProfile::CompilerOperations
    ) && primitive_method(rows, headers, header, work)?;
    Ok(primitive
        || profile.conversions()
            && (conversions::method(
                rows,
                headers,
                header,
                roles.languages,
                Constness::Ordinary,
                work,
            )? || bodies
                && (captured::method(rows, headers, header, work)?
                    || reservation_parent(rows, headers, header, work)?.is_some())))
}

fn reservation_parent<'a>(
    rows: &'a SourceRows,
    headers: &HeaderIndex<'_>,
    header: &super::super::FunctionHeader,
    work: &mut Work,
) -> Result<Option<&'a super::super::ImplHeader>, &'static str> {
    let Some(parent) = header.parent else {
        return Ok(None);
    };
    let Some(super::super::HeaderRef::Implementation(index)) = headers.header(parent, work)? else {
        return Ok(None);
    };
    Ok(rows
        .implementations
        .get(index as usize)
        .filter(|implementation| {
            implementation.self_definition == parent
                && implementation.polarity == ImplPolarity::Reservation
        }))
}
fn trait_const_method(
    rows: &SourceRows,
    headers: &HeaderIndex<'_>,
    header: &super::super::FunctionHeader,
    roles: &Bindings,
    work: &mut Work,
) -> Result<bool, &'static str> {
    let Some(parent) = header
        .parent
        .filter(|_| header.constness == Constness::Const)
    else {
        return Ok(false);
    };
    let Some(super::super::HeaderRef::Implementation(index)) = headers.header(parent, work)? else {
        return Ok(false);
    };
    let implementation = &rows.implementations[index as usize];
    if implementation.self_definition != parent || !implementation.constant {
        return Ok(false);
    }
    if implementation.polarity == ImplPolarity::Reservation {
        return Ok(true);
    }
    Ok(
        roles.const_binary_impl(rows, headers, implementation, work)?
            && conversions::method(rows, headers, header, false, Constness::Const, work)?,
    )
}

/// Version thirteen: an inherent method may be `const`. Its original is then
/// a const definition re-checked by every consumer, as a free one is.
fn inherent_const(
    rows: &SourceRows,
    headers: &HeaderIndex<'_>,
    header: &super::super::FunctionHeader,
    work: &mut Work,
) -> Result<bool, &'static str> {
    let Some(parent) = header
        .parent
        .filter(|_| header.constness == Constness::Const)
    else {
        return Ok(false);
    };
    let Some(super::super::HeaderRef::Implementation(index)) = headers.header(parent, work)? else {
        return Ok(false);
    };
    Ok(rows
        .implementations
        .get(index as usize)
        .is_some_and(|implementation| implementation.trait_application.is_none()))
}
/// `bodies` is version thirteen, whose free originals may also bind late
/// region formals; every consumer re-proves them in its rebuilt environment.
fn functions(
    rows: &SourceRows,
    headers: &HeaderIndex<'_>,
    work: &mut Work,
    profile: NamespaceProfile,
    roles: &Bindings,
    bodies: bool,
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
        let free_effects = profile.callables() && header.parent.is_none();
        // Version thirteen: an associated function may also be unsafe. Its
        // constness, convention and variadic forms keep their fences.
        let safety = free_effects || (bodies && header.parent.is_some());
        let callable = callable_roles::required_member(header, roles.callables, work)?;
        let required = callable
            || profile.conversions() && roles.required_member(rows, headers, header, work)?;
        let inherent_const = bodies && inherent_const(rows, headers, header, work)?;
        let trait_const =
            bodies && !inherent_const && trait_const_method(rows, headers, header, roles, work)?;
        let supported_signature = |signature: super::super::Signature| {
            (safety || !signature.unsafe_)
                && !signature.variadic
                && (callable
                    || admitted_convention(rows, header.declaration, signature.convention, profile))
        };
        if (!free_effects
            && !required
            && !inherent_const
            && !trait_const
            && header.constness != Constness::Ordinary)
            || !required && header.constness == Constness::Conditional
            || !supported_signature(header.original)
            || !supported_signature(header.callable)
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
        if bodies && header.parent.is_some() && header.original.unsafe_ != header.callable.unsafe_ {
            return Err("source associated callable changed its actual original/public safety");
        }
        let associated =
            trait_const || associated_method(rows, headers, header, work, profile, roles, bodies)?;
        if matches!(header.body, FunctionBody::Original(_))
            && header.parent.is_some()
            && !associated
        {
            return Err("source provided associated bodies require a wider explicit profile");
        }
        let late_free = bodies
            && header.parent.is_none()
            && header.regions.captures == 0
            && header.regions.early.start == header.regions.early.end;
        if matches!(header.body, FunctionBody::Original(_))
            && header.regions.total != 0
            && !(profile.conversions() && (associated || late_free))
        {
            return Err("source original region formals require their complete wider body profile");
        }
        let parent = rows.declarations[header.declaration.0 as usize]
            .definition
            .parent
            .ok_or("source function lacks its actual lexical declaration owner")?;
        let parent_kind = rows.declarations[parent.0 as usize].definition.kind;
        match header.body {
            FunctionBody::Absent
                if (header.parent != Some(parent)
                    || parent_kind != super::super::DefinitionKind::Trait)
                    && !rows.bodyless_runtime(header.declaration, work)? =>
            {
                return Err(
                    "source body absence is supported only for actual required trait members",
                );
            }
            FunctionBody::Original(_)
                if parent_kind != super::super::DefinitionKind::Module
                    && !associated
                    && !(bodies
                        && profile == NamespaceProfile::CompilerOperations
                        && super::nested_original(&rows.declarations, header).is_some()) =>
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
        || !implementation.polarity.is_positive()
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
    profile: NamespaceProfile,
    binding: BindingProfile,
    features: bool,
) -> Result<(), &'static str> {
    let regional = profile.conversions();
    let constant_initializers = binding == BindingProfile::BodyOperations;
    for original in &rows.originals {
        work.charge(1)?;
        let expected = if profile.callables() {
            callable_traits::original_context(rows, headers, original, work, constant_initializers)?
        } else {
            super::super::CallContext::Runtime
        };
        if original.context != expected
            || !regional && !original.variables.is_empty()
            || !regional && !original.projections.is_empty()
            || !features && (!original.calls.is_empty() || !original.arguments.is_empty())
            || !regional && !original.openings.is_empty()
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
                    ValueKind::Function(_)
                        | ValueKind::Call { .. }
                        | ValueKind::Scalar(_)
                        | ValueKind::Aggregate { .. }
                )
                || regional
                    && matches!(
                        value.kind,
                        ValueKind::Borrow(_)
                            | ValueKind::TwoPhaseBorrow(_)
                            | ValueKind::Read(Place::Projection(_) | Place::Temporary(_))
                            | ValueKind::ScalarCast(_)
                            | ValueKind::Binary { .. }
                            | ValueKind::Invoke { .. }
                            | ValueKind::VariantAggregate { .. }
                            | ValueKind::IsVariant { .. }
                            | ValueKind::EnumCast(_)
                            | ValueKind::Text(_)
                            | ValueKind::Reify(_)
                            | ValueKind::PointerCall { .. }
                            | ValueKind::Assembly { .. }
                            | ValueKind::Constant(_)
                            | ValueKind::AssociatedConstant { .. }
                    )
                || constant_initializers && matches!(value.kind, ValueKind::ConstArgument(_)))
            {
                return Err("source value operation requires a wider explicit profile");
            }
        }
        for statement in &original.statements {
            work.charge(1)?;
            if !regional
                && matches!(
                    statement.action,
                    Action::Project(_)
                        | Action::Store {
                            place: Place::Projection(_),
                            ..
                        }
                        | Action::CheckInitialized(Place::Projection(_))
                        | Action::Mention(Place::Projection(_))
                )
            {
                return Err("source projected operation requires a wider explicit profile");
            }
        }
    }
    Ok(())
}
/// Recorded preparation inputs: `scoped` admits only the vacuous formation
/// scope that version-thirteen parenthesized callable sugar opens.
fn body_inputs(
    rows: &SourceRows,
    work: &mut Work,
    regional: bool,
    scoped: bool,
) -> Result<(), &'static str> {
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
                ordinary(&request.value, false)?;
            }
        }
        for request in &input.type_assumptions {
            work.charge(1)?;
            ordinary(&request.value, false)?;
        }
        normalization_ownership(input, work, regional)?;
        for obligation in &input.obligations {
            work.charge(1)?;
            // Parenthesized callable sugar opens a clause binder even when it
            // binds no region; only that vacuous formation scope is admitted.
            let vacuous = scoped
                && matches!(obligation.kind, InputKind::ScopedWellFormed { frame, .. }
                    if rows.binders.get(frame.0 as usize).is_some_and(|b| b.count == 0));
            if !vacuous
                && matches!(
                    obligation.kind,
                    InputKind::Membership { .. } | InputKind::ScopedWellFormed { .. }
                )
                || !regional
                    && matches!(
                        obligation.kind,
                        InputKind::Variable { .. } | InputKind::LocalBorrow(_)
                    )
            {
                return Err("source allocation/scoped input requires a wider explicit profile");
            }
        }
    }
    Ok(())
}
/// Owned normalization is data only; the recorded body domain proves each slot.
fn normalization_ownership(
    input: &super::super::BodyInputs,
    work: &mut Work,
    regional: bool,
) -> Result<(), &'static str> {
    for normalization in &input.normalizations {
        work.charge(1)?;
        if !normalization.owned.is_empty()
            && (!regional
                || !matches!(normalization.owned.as_slice(), [range] if range.start < range.end))
        {
            return Err("source owned normalization requires a wider explicit profile");
        }
    }
    Ok(())
}
fn recipes(
    rows: &SourceRows,
    work: &mut Work,
    features: bool,
    regional: bool,
) -> Result<(), &'static str> {
    work.charge(rows.function_applications.len())?;
    if !features && !rows.function_applications.is_empty() {
        return Err("source application recipes require a wider source profile");
    }
    for row in &rows.binders {
        work.charge(1)?;
        if matches!(row.kind, super::super::BinderKind::CallSignature { .. })
            && (!features || !regional && row.count != 0)
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
        if !regional && matches!(row, super::super::Region::Variable { .. }) {
            return Err("source body allocation recipe requires a wider source profile");
        }
    }
    Ok(())
}
/// `callable` admits the conditional hierarchy of an actual callable role.
fn ordinary(request: &super::super::Requirement, callable: bool) -> Result<(), &'static str> {
    if matches!(request, super::super::Requirement::Outlives(_)) {
        return Err("standalone regional clauses require their explicit wider header consumer");
    }
    if matches!(request, super::super::Requirement::Trait { effect, .. } if *effect != Constness::Ordinary && !(callable && *effect == Constness::Conditional))
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
    Conversions,
    CompilerOperations,
    /// Version thirteen: every spelled trait role; consumers prove their use.
    Languages,
    /// Version fourteen combines general language headers and exact operation contracts.
    BodyOperations,
}
impl BindingProfile {
    fn languages(self) -> bool {
        matches!(self, Self::Languages | Self::BodyOperations)
    }
    fn definition_policies(self) -> u16 {
        if self == Self::BodyOperations {
            TraitFlags::DECLARATION_POLICIES
        } else {
            0
        }
    }
}
pub(super) struct Bindings {
    sizing: [Option<crate::support::crates::owner_catalog::DeclarationRef>; 3],
    destruct: Option<crate::support::crates::owner_catalog::DeclarationRef>,
    freeze: Option<crate::support::crates::owner_catalog::DeclarationRef>,
    no_drop: Option<crate::support::crates::owner_catalog::DeclarationRef>,
    conversion: Option<crate::support::crates::owner_catalog::DeclarationRef>,
    binary_operators: [Option<crate::support::crates::owner_catalog::DeclarationRef>;
        Profile::CONST_BINARY_ROLES.len()],
    range: Option<interval_roles::Family>,
    callables: callable_roles::Roles,
    /// Version thirteen: exact empty auto declarations, unsafe traits and
    /// every trait impl's own safety and polarity.
    languages: bool,
    definition_policies: u16,
    /// The role-bound auto markers, in `callable_traits::MARKERS` order.
    markers: [Option<crate::support::crates::owner_catalog::DeclarationRef>; 4],
}
impl Bindings {
    fn const_binary_impl(
        &self,
        rows: &SourceRows,
        headers: &HeaderIndex<'_>,
        implementation: &super::super::ImplHeader,
        work: &mut Work,
    ) -> Result<bool, &'static str> {
        let Some(application) = implementation.trait_application else {
            return Ok(false);
        };
        if !implementation.constant
            || implementation.unsafe_
            || !implementation.polarity.is_positive()
            || self.binary_operators.iter().all(Option::is_none)
        {
            return Ok(false);
        }
        work.charge(1)?;
        let declaration = rows
            .trait_applications
            .get(application.0 as usize)
            .ok_or("source const operator impl lost its actual trait application")?
            .declaration;
        if declaration.artifact.0 != 0
            || !self
                .binary_operators
                .contains(&Some(declaration.declaration))
        {
            return Ok(false);
        }
        let Some(super::super::HeaderRef::Trait(index)) =
            headers.header(declaration.declaration, work)?
        else {
            return Err("source const operator impl lost its actual trait header");
        };
        Ok(rows.traits[index as usize].flags.bits() & TraitFlags::CONST != 0)
    }

    fn required_member(
        &self,
        rows: &SourceRows,
        headers: &HeaderIndex<'_>,
        function: &super::super::FunctionHeader,
        work: &mut Work,
    ) -> Result<bool, &'static str> {
        if self
            .range
            .is_some_and(|family| family.method == function.declaration)
        {
            return Ok(true);
        }
        let binary = function
            .parent
            .filter(|&owner| self.binary_operators.contains(&Some(owner)));
        conversions::required_member(
            rows,
            headers,
            function,
            binary.or(self.conversion),
            binary.is_some(),
            work,
        )
    }

    fn marker(
        &self,
        declaration: crate::support::crates::owner_catalog::DeclarationRef,
    ) -> Option<callable_traits::Marker> {
        self.markers
            .iter()
            .zip(callable_traits::MARKERS)
            .find_map(|(bound, marker)| (*bound == Some(declaration)).then_some(marker))
    }
}
fn role_allowed(profile: BindingProfile, role: LangItem) -> bool {
    if matches!(
        role,
        LangItem::TypeInfo
            | LangItem::TypeId
            | LangItem::PointeeTrait
            | LangItem::MetadataType
            | LangItem::DynMetadata
            | LangItem::ConstParamTy
            | LangItem::StructuralPartialEq
            | LangItem::PanicLocation
            | LangItem::PanicInfo
            | LangItem::FormatArguments
            | LangItem::FormatArgument
            | LangItem::MemSizeConst
            | LangItem::MemAlignConst
    ) || interval_roles::ROLES.contains(&role)
        || LangItem::NOMINAL_WRAPPERS.contains(&role)
        || role == LangItem::BikeshedGuaranteedNoDrop
        || LangItem::GUARD_PANICS.contains(&role)
        || role.is_runtime_import()
    {
        return profile == BindingProfile::BodyOperations;
    }
    if matches!(role, LangItem::UnwindSafe | LangItem::RefUnwindSafe) {
        return false;
    }
    if matches!(
        role,
        LangItem::Start | LangItem::Termination | LangItem::BeginPanic | LangItem::EhPersonality
    ) {
        return false;
    }
    if profile.languages() {
        // General language headers alone retain the v13 future fence. The new
        // operation profile adds precisely v11's future family; Send stays closed.
        return role != LangItem::Send
            && (profile == BindingProfile::BodyOperations
                || !matches!(
                    role,
                    LangItem::Future
                        | LangItem::FutureOutput
                        | LangItem::FuturePoll
                        | LangItem::AsyncDropInPlace
                ));
    }
    matches!(
        role,
        LangItem::Sized
            | LangItem::MetaSized
            | LangItem::PointeeSized
            | LangItem::LegacyReceiver
            | LangItem::Clone
            | LangItem::CloneFn
            | LangItem::Copy
    ) || (matches!(
        profile,
        BindingProfile::Provided
            | BindingProfile::Callables
            | BindingProfile::Conversions
            | BindingProfile::CompilerOperations
    ) && matches!(role, LangItem::Destruct | LangItem::Freeze))
        || matches!(
            profile,
            BindingProfile::Conversions | BindingProfile::CompilerOperations
        ) && role == LangItem::From
        || profile == BindingProfile::CompilerOperations
            && matches!(
                role,
                LangItem::Future
                    | LangItem::FutureOutput
                    | LangItem::FuturePoll
                    | LangItem::AsyncDropInPlace
            )
}
fn binding_declarations(
    rows: &SourceRows,
    headers: &HeaderIndex<'_>,
    work: &mut Work,
    profile: BindingProfile,
) -> Result<
    [Option<crate::support::crates::owner_catalog::DeclarationRef>; LangItem::ALL.len()],
    &'static str,
> {
    work.charge(1)?;
    if rows.policy.roles.start != 0 || rows.policy.roles.end as usize != rows.roles.len() {
        return Err("source role policy omits part of its exact binding inventory");
    }
    let mut declarations = [None; LangItem::ALL.len()];
    for role in &rows.roles {
        work.charge(declarations.len())?;
        let allowed = role_allowed(profile, role.role);
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
        let declaration = rows
            .declarations
            .get(address.declaration.0 as usize)
            .ok_or("source role binding names an unknown declaration")?;
        role_header(
            profile,
            rows,
            declaration.definition.kind,
            role.role,
            headers.header(address.declaration, work)?,
        )?;
        declarations[role.role.index()] = Some(address.declaration);
    }
    Ok(declarations)
}

pub(super) fn bindings(
    rows: &SourceRows,
    headers: &HeaderIndex<'_>,
    work: &mut Work,
    profile: BindingProfile,
) -> Result<Bindings, &'static str> {
    let declarations = binding_declarations(rows, headers, work, profile)?;
    let send = diagnostic_bindings(rows, headers, work, profile, &declarations)?;
    owned_role_members(rows, headers, work, &declarations)?;
    let sizing = [
        declarations[LangItem::Sized.index()],
        declarations[LangItem::MetaSized.index()],
        declarations[LangItem::PointeeSized.index()],
    ];
    if sizing.contains(&None) {
        return Err("source functions require the real complete sizing trio");
    }
    slot_bindings(rows, headers, work, profile, sizing[0])?;
    let callables = if profile.languages() {
        callable_roles::Roles(
            [
                LangItem::Tuple,
                LangItem::FnOnce,
                LangItem::FnMut,
                LangItem::Fn,
            ]
            .map(|role| declarations[role.index()]),
        )
    } else {
        callable_roles::Roles::default()
    };
    let languages = profile.languages();
    let range = if profile == BindingProfile::BodyOperations {
        interval_roles::family(
            rows,
            headers,
            interval_roles::ROLES.map(|role| declarations[role.index()]),
            work,
        )?
    } else {
        None
    };
    Ok(Bindings {
        callables,
        languages,
        range,
        definition_policies: profile.definition_policies(),
        markers: if languages {
            [
                send,
                declarations[LangItem::Sync.index()],
                declarations[LangItem::Unpin.index()],
                declarations[LangItem::UnsafeUnpin.index()],
            ]
        } else {
            [None; 4]
        },
        conversion: (matches!(
            profile,
            BindingProfile::Conversions
                | BindingProfile::CompilerOperations
                | BindingProfile::Languages
                | BindingProfile::BodyOperations
        ))
        .then_some(declarations[LangItem::From.index()])
        .flatten(),
        binary_operators: if profile == BindingProfile::BodyOperations {
            Profile::CONST_BINARY_ROLES.map(|role| declarations[role.index()])
        } else {
            [None; Profile::CONST_BINARY_ROLES.len()]
        },
        sizing,
        destruct: if matches!(
            profile,
            BindingProfile::Callables
                | BindingProfile::Conversions
                | BindingProfile::Languages
                | BindingProfile::BodyOperations
        ) {
            declarations[LangItem::Destruct.index()]
        } else {
            None
        },
        no_drop: declarations[LangItem::BikeshedGuaranteedNoDrop.index()],
        freeze: if matches!(
            profile,
            BindingProfile::Callables
                | BindingProfile::Conversions
                | BindingProfile::Languages
                | BindingProfile::BodyOperations
        ) {
            declarations[LangItem::Freeze.index()]
        } else {
            None
        },
    })
}

/// Version thirteen's diagnostic identities: each names, once, one of this
/// provider's own declarations. `NonNull` names a struct nominal, as the local
/// rule requires; `Send` names an unsafe auto trait, whose exact empty
/// declaration `trait_headers` re-proves. Returns `Send`'s declaration.
fn diagnostic_bindings(
    rows: &SourceRows,
    headers: &HeaderIndex<'_>,
    work: &mut Work,
    profile: BindingProfile,
    declarations: &[Option<crate::support::crates::owner_catalog::DeclarationRef>],
) -> Result<Option<crate::support::crates::owner_catalog::DeclarationRef>, &'static str> {
    let diagnostics = rows.diagnostics();
    if !diagnostics.is_empty() && !profile.languages() {
        return Err("source diagnostic identity requires a language role table");
    }
    let mut seen = [false; super::super::DiagnosticRole::ALL.len()];
    let mut send = None;
    for row in diagnostics {
        work.charge(1)?;
        if row.declaration.artifact != super::super::ArtifactRef(0) {
            return Err("source diagnostic identity belongs to a foreign provider");
        }
        let slot = &mut seen[row.role as usize];
        if *slot {
            return Err("duplicate source diagnostic identity");
        }
        *slot = true;
        let kind = rows
            .declarations
            .get(row.declaration.declaration.0 as usize)
            .map(|declaration| declaration.definition.kind);
        let header = headers.header(row.declaration.declaration, work)?;
        match row.role {
            super::super::DiagnosticRole::NonNull => {
                if kind != Some(super::super::DefinitionKind::Struct)
                    || !matches!(header, Some(super::super::HeaderRef::Nominal(_)))
                {
                    return Err(
                        "imported NonNull diagnostic item requires its nominal declaration profile",
                    );
                }
            }
            super::super::DiagnosticRole::Send => {
                let Some(super::super::HeaderRef::Trait(index)) = header else {
                    return Err(SEND);
                };
                if kind != Some(super::super::DefinitionKind::Trait)
                    || rows
                        .traits
                        .get(index as usize)
                        .map(|t| t.flags.bits() & !profile.definition_policies())
                        != Some(TraitFlags::UNSAFE | TraitFlags::AUTO)
                {
                    return Err(SEND);
                }
                send = Some(row.declaration.declaration);
            }
            super::super::DiagnosticRole::Eq => {
                if profile != BindingProfile::BodyOperations
                    || kind != Some(super::super::DefinitionKind::Trait)
                    || !matches!(header, Some(super::super::HeaderRef::Trait(_)))
                    || declarations[LangItem::StructuralPartialEq.index()]
                        == Some(row.declaration.declaration)
                    || declarations[LangItem::ConstParamTy.index()]
                        == Some(row.declaration.declaration)
                {
                    return Err(
                        "source Eq diagnostic identity requires its own version-fourteen trait",
                    );
                }
            }
        }
    }
    let const_marker = declarations[LangItem::ConstParamTy.index()].is_some();
    if seen[super::super::DiagnosticRole::Eq as usize] != const_marker
        || const_marker && declarations[LangItem::StructuralPartialEq.index()].is_none()
    {
        return Err(
            "source ConstParamTy requires its actual StructuralPartialEq and Eq declarations",
        );
    }
    Ok(send)
}
const SEND: &str = "imported Send diagnostic item requires its unsafe auto trait declaration";
/// The declaration/header kind each bound role may claim under `profile`.
fn role_header(
    profile: BindingProfile,
    rows: &SourceRows,
    kind: super::super::DefinitionKind,
    role: LangItem,
    header: Option<super::super::HeaderRef>,
) -> Result<(), &'static str> {
    match (role, header) {
        (role, Some(super::super::HeaderRef::Function(index)))
            if profile == BindingProfile::BodyOperations
                && LangItem::GUARD_PANICS.contains(&role)
                && rows.functions.get(index as usize).is_some_and(|function| {
                    function.parent.is_none() && matches!(function.body, FunctionBody::Original(_))
                }) => {}
        (role, _) if LangItem::GUARD_PANICS.contains(&role) => {
            return Err("source guard role requires its actual defined free function");
        }
        (role, Some(super::super::HeaderRef::Function(_)))
            if role.is_runtime_import() && profile == BindingProfile::BodyOperations => {}
        (role, Some(super::super::HeaderRef::Nominal(index)))
            if LangItem::NOMINAL_WRAPPERS.contains(&role)
                && profile == BindingProfile::BodyOperations
                && wrapper_header(rows, kind, role, index) => {}
        (role, _) if LangItem::NOMINAL_WRAPPERS.contains(&role) => {
            return Err("source wrapper role requires its actual nominal header");
        }
        (LangItem::PointeeTrait, Some(super::super::HeaderRef::Trait(_)))
        | (LangItem::MetadataType, Some(super::super::HeaderRef::AssociatedType(_)))
        | (
            LangItem::RangeMin
            | LangItem::RangeMax
            | LangItem::MemSizeConst
            | LangItem::MemAlignConst,
            Some(super::super::HeaderRef::Constant(_)),
        )
        | (LangItem::RangeSub, Some(super::super::HeaderRef::Function(_)))
        | (
            LangItem::DynMetadata | LangItem::TypeInfo | LangItem::TypeId | LangItem::PanicLocation,
            Some(super::super::HeaderRef::Nominal(_)),
        ) if profile == BindingProfile::BodyOperations => {}
        (LangItem::PointeeTrait | LangItem::MetadataType | LangItem::DynMetadata, _) => {
            return Err("source Pointee roles require their actual declaration header kinds");
        }
        (LangItem::RangeMin | LangItem::RangeMax | LangItem::RangeSub, _) => {
            return Err("source range roles require their actual associated header kinds");
        }
        (LangItem::TypeInfo | LangItem::TypeId, _) => {
            return Err("source reflection role requires its actual struct header");
        }
        (
            LangItem::PanicInfo | LangItem::FormatArguments | LangItem::FormatArgument,
            Some(super::super::HeaderRef::Nominal(_)),
        ) if profile == BindingProfile::BodyOperations
            && kind == super::super::DefinitionKind::Struct => {}
        (LangItem::PanicLocation, _) => {
            return Err("source caller location role requires its actual nominal header");
        }
        (LangItem::PanicInfo | LangItem::FormatArguments | LangItem::FormatArgument, _) => {
            return Err("source panic or formatting role requires its actual struct header");
        }
        (
            LangItem::CloneFn | LangItem::FuturePoll | LangItem::AsyncDropInPlace,
            Some(super::super::HeaderRef::Function(_)),
        ) => {}
        (LangItem::FutureOutput, Some(super::super::HeaderRef::AssociatedType(_)))
            if matches!(
                profile,
                BindingProfile::CompilerOperations | BindingProfile::BodyOperations
            ) => {}
        // A bound `panic` names a free function; consumers re-prove its header.
        // A bound `phantom_data` names a nominal; consumers re-prove its unit struct.
        (LangItem::FnOnceOutput, Some(super::super::HeaderRef::AssociatedType(_)))
        | (LangItem::Panic, Some(super::super::HeaderRef::Function(_)))
        | (LangItem::PhantomData, Some(super::super::HeaderRef::Nominal(_)))
            if profile.languages() => {}
        (
            _,
            Some(
                super::super::HeaderRef::AssociatedType(_)
                | super::super::HeaderRef::AssociatedDefinition(_),
            ),
        ) => {
            return Err("source associated role has a different actual member kind");
        }
        (LangItem::CloneFn, _)
        | (
            _,
            None
            | Some(
                super::super::HeaderRef::Function(_)
                | super::super::HeaderRef::Implementation(_)
                | super::super::HeaderRef::Static(_)
                | super::super::HeaderRef::TypeAlias(_)
                | super::super::HeaderRef::TraitAlias(_)
                | super::super::HeaderRef::Constant(_),
            ),
        ) => return Err("source role has a different actual declaration/header kind"),
        (_, Some(super::super::HeaderRef::Trait(_))) => {}
        (
            _,
            Some(
                super::super::HeaderRef::Nominal(_)
                | super::super::HeaderRef::Variant(_)
                | super::super::HeaderRef::Constructor(_),
            ),
        ) => {
            return Err("ordinary source roles cannot claim a nominal declaration");
        }
    }
    Ok(())
}
fn wrapper_header(
    rows: &SourceRows,
    kind: super::super::DefinitionKind,
    role: LangItem,
    index: u32,
) -> bool {
    use super::super::{DefinitionKind, NominalForm};
    let expected = if role == LangItem::CVoid {
        DefinitionKind::Enum
    } else {
        DefinitionKind::Struct
    };
    kind == expected
        && rows.nominals.get(index as usize).is_some_and(|nominal| {
            (nominal.layout.form == NominalForm::Union) == (role == LangItem::MaybeUninit)
        })
}
/// `clone_fn` and `fn_once_output` are owned by their exact bound parents.
fn owned_role_members(
    rows: &SourceRows,
    headers: &HeaderIndex<'_>,
    work: &mut Work,
    declarations: &[Option<crate::support::crates::owner_catalog::DeclarationRef>;
         LangItem::ALL.len()],
) -> Result<(), &'static str> {
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
    if let Some(output) = declarations[LangItem::FnOnceOutput.index()] {
        let Some(super::super::HeaderRef::AssociatedType(index)) = headers.header(output, work)?
        else {
            return Err("source fn_once_output has no exact declared result header");
        };
        if Some(rows.associated_types[index as usize].parent)
            != declarations[LangItem::FnOnce.index()]
        {
            return Err("source fn_once_output belongs to another actual FnOnce declaration");
        }
    }
    let pointee = declarations[LangItem::PointeeTrait.index()];
    let metadata = declarations[LangItem::MetadataType.index()];
    let dyn_metadata = declarations[LangItem::DynMetadata.index()];
    if pointee.is_some() || metadata.is_some() || dyn_metadata.is_some() {
        let (Some(pointee), Some(metadata)) = (pointee, metadata) else {
            return Err("source Pointee roles require both actual trait and associated roles");
        };
        let Some(super::super::HeaderRef::AssociatedType(index)) =
            headers.header(metadata, work)?
        else {
            return Err("source Metadata has no exact associated header");
        };
        if rows.associated_types[index as usize].parent != pointee {
            return Err("source Metadata belongs to another actual Pointee declaration");
        }
    }
    layout_members(
        rows,
        headers,
        work,
        [
            declarations[LangItem::MemSizeConst.index()],
            declarations[LangItem::MemAlignConst.index()],
        ],
    )?;
    Ok(())
}

fn layout_members(
    rows: &SourceRows,
    headers: &HeaderIndex<'_>,
    work: &mut Work,
    members: [Option<crate::support::crates::owner_catalog::DeclarationRef>; 2],
) -> Result<(), &'static str> {
    let mut shared = None;
    for member in members.into_iter().flatten() {
        work.charge(4)?;
        let parent = rows.declarations[member.0 as usize]
            .definition
            .parent
            .ok_or("source layout constant requires its actual trait owner")?;
        let Some(super::super::HeaderRef::Trait(index)) = headers.header(parent, work)? else {
            return Err("source layout constant requires its actual trait header");
        };
        let actual = rows.traits[index as usize].members.slice(&rows.members)?;
        work.charge(actual.len())?;
        if !actual.contains(&member) || shared.replace(parent).is_some_and(|owner| owner != parent)
        {
            return Err("source layout constants changed their actual trait member ownership");
        }
        let Some(super::super::HeaderRef::Constant(index)) = headers.header(member, work)? else {
            return Err("source layout constant lacks its actual default header");
        };
        let constant = &rows.constants()[index as usize];
        if !matches!(constant.body, super::super::ConstantBody::Original(_))
            || rows.types.get(constant.ty.0 as usize)
                != Some(&super::super::Type::Scalar(
                    crate::support::crates::Scalar::Usize,
                ))
        {
            return Err("source layout constant requires its actual provided usize default");
        }
    }
    Ok(())
}

/// Sized formals name the actual Sized role; a version-thirteen default is
/// admitted only as its owning trait's Self.
fn slot_bindings(
    rows: &SourceRows,
    headers: &HeaderIndex<'_>,
    work: &mut Work,
    profile: BindingProfile,
    sized_role: Option<crate::support::crates::owner_catalog::DeclarationRef>,
) -> Result<(), &'static str> {
    for slot in &rows.slots {
        work.charge(1)?;
        // Version thirteen's const slot carries its type and nothing else.
        if matches!(slot.kind, super::super::ParameterKind::Const { .. }) && slot.default.is_some()
        {
            return Err("source const parameter slot has a default");
        }
        if let Some(default) = slot.default {
            let Some(super::super::HeaderRef::Trait(index)) = headers.header(slot.owner, work)?
            else {
                return Err("source parameter default belongs to a non-trait owner");
            };
            if !profile.languages() || rows.traits[index as usize].self_type != default {
                return Err(
                    "source parameter default other than its trait's Self requires a wider consumer",
                );
            }
        }
        if let super::super::ParameterKind::Type {
            sized: Some(address),
        } = slot.kind
            && (address.artifact != super::super::ArtifactRef(0)
                || Some(address.declaration) != sized_role)
        {
            return Err("source sized formal names a foreign or different actual Sized role");
        }
    }
    Ok(())
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
            NamespaceProfile::Definitions
                | NamespaceProfile::Callables
                | NamespaceProfile::Safety
                | NamespaceProfile::Conversions
                | NamespaceProfile::CompilerOperations
        ) && convention == Convention::C
            && rows
                .exact_callables
                .binary_search_by_key(&declaration.0, |row| row.declaration.0)
                .is_ok())
}

#[cfg(test)]
#[path = "../../../../tests/invariants/valued_slots.rs"]
mod const_tests;
