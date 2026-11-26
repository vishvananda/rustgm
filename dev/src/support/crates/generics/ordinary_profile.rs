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
#[path = "iterator_blanket.rs"]
mod iterator_blanket;
#[path = "logical_discriminant.rs"]
mod logical_discriminant;
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
        (BindingProfile::Languages, false),
    )
}
pub(super) fn validate_body_operations(
    rows: &SourceRows,
    headers: &HeaderIndex<'_>,
    work: &mut Work,
    profile: Profile,
) -> Result<(), &'static str> {
    validate_with(
        rows,
        headers,
        work,
        NamespaceProfile::CompilerOperations,
        (BindingProfile::BodyOperations, profile.bootstrap_bindings()),
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
    validate_with(rows, headers, work, profile, (roles, false))
}
fn validate_with(
    rows: &SourceRows,
    headers: &HeaderIndex<'_>,
    work: &mut Work,
    profile: NamespaceProfile,
    (binding, bootstrap): (BindingProfile, bool),
) -> Result<(), &'static str> {
    work.charge(1)?;
    if !profile.conversions() && !rows.body_units.is_empty() {
        return Err("source numbering units require their wider explicit profile");
    }
    if rows.policy.edition != Edition::Rust2024 || rows.policy.provider.0 != 0 {
        return Err("ordinary source functions require their own Rust2024 provider profile");
    }
    let roles = bindings_with_policy(rows, headers, work, binding, bootstrap)?;
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
        let inherent_const_impl = binding == BindingProfile::BodyOperations
            && header.trait_application.is_none()
            && header.polarity.is_positive();
        if header.constant
            && !reservation
            && !inherent_const_impl
            && roles
                .const_trait_impl(rows, headers, header, work)?
                .is_none()
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
    let modern = binding == BindingProfile::BodyOperations;
    for clause in &rows.clauses {
        work.charge(1)?;
        let qualified = modern
            || roles.callables.callable(clause.owner)
            || roles.conditional_role_input(rows, clause)
            || roles.conditional_iterator_input(rows, headers, clause, work)?;
        ordinary(&clause.requirement, qualified, modern)?;
    }
    originals(
        rows,
        headers,
        work,
        profile,
        (binding, bootstrap),
        features,
        &roles,
    )?;
    body_inputs(
        rows,
        headers,
        work,
        profile.conversions(),
        binding.languages(),
        roles.default_traits(),
        binding == BindingProfile::BodyOperations,
    )
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
        let role_header = super::super::TraitHeader {
            flags: trait_profile_flags(header, roles)?,
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
        if roles.unsize == Some(header.declaration) {
            require_unsize_header(header)?;
            continue;
        }
        if let Some(restricted) = roles.restricted_trait(header.declaration, flags) {
            require_restricted_header(rows, header, restricted)?;
            continue;
        }
        if roles.transmute == Some(header.declaration) {
            require_transmute_header(header)?;
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
        if conversion {
            conversions::trait_header(rows, headers, header, work)?;
        }
        require_ordinary_trait_flags(roles, header.declaration, flags, conversion)?;
        // The sealed discriminator role owns its exact deny-impl and dyn
        // restrictions independently of the sizing declarations.
        if roles.discriminant != Some(header.declaration) {
            ordinary_sizing_policy(header.declaration, flags, role_header.flags, sizing)?;
        }
    }
    Ok(())
}

fn require_ordinary_trait_flags(
    roles: &Bindings,
    declaration: crate::support::crates::owner_catalog::DeclarationRef,
    flags: u16,
    conversion: bool,
) -> Result<(), &'static str> {
    let const_role = conversion
        || roles.const_operator(declaration)
        || roles
            .range
            .is_some_and(|family| family.owner == declaration)
        || roles.try_traits.contains(&Some(declaration))
        || roles.for_traits.contains(&Some(declaration))
        || [roles.drop, roles.clone, roles.trivial_clone].contains(&Some(declaration));
    // Version thirteen also admits an unsafe trait that is not auto.
    if flags & TraitFlags::AUTO != 0
        || flags & TraitFlags::UNSAFE != 0 && !roles.languages
        || flags & TraitFlags::CONST != 0 && roles.definition_policies == 0 && !const_role
    {
        return Err("source const/auto/unsafe traits require a wider explicit profile");
    }
    Ok(())
}

fn trait_profile_flags(
    header: &super::super::TraitHeader,
    roles: &Bindings,
) -> Result<TraitFlags, &'static str> {
    let flags = header.flags.bits();
    let policies = roles.definition_policies;
    if flags & TraitFlags::UNSAFE_SPECIALIZATION != 0 {
        if policies == 0 {
            return Err("source unsafe specialization requires the declaration policy profile");
        }
        if roles.trivial_clone == Some(header.declaration) && flags & TraitFlags::MARKER == 0 {
            return Err("source unsafe specialization requires its actual TrivialClone marker");
        }
    }
    if header.region_captures != 0 && policies == 0 {
        return Err("source trait lifetimes require the declaration policy profile");
    }
    if flags & TraitFlags::INCOHERENT_INHERENT & !policies != 0 {
        return Err("source incoherent trait requires the declaration policy profile");
    }
    if flags & TraitFlags::DISPATCH_POLICIES & !policies != 0 {
        return Err("source trait dispatch requires the declaration policy profile");
    }
    if flags & TraitFlags::MARKER != 0
        && (policies & TraitFlags::MARKER == 0 || header.members.start != header.members.end)
    {
        return Err("source marker trait requires its actual empty declaration profile");
    }
    let general_specialization =
        if policies != 0 && !roles.sizing.contains(&Some(header.declaration)) {
            TraitFlags::SPECIALIZATION
        } else {
            0
        };
    TraitFlags::from_bits(flags & !(policies | general_specialization))
}

fn require_unsize_header(header: &super::super::TraitHeader) -> Result<(), &'static str> {
    if header.flags.bits() != (TraitFlags::DENY_IMPL | TraitFlags::DYN_INCOMPATIBLE)
        || header.written_parameters != 1
        || header.region_captures != 0
        || header.members.start != header.members.end
    {
        return Err("source Unsize requires its empty restricted trait declaration");
    }
    Ok(())
}

fn require_restricted_header(
    rows: &SourceRows,
    header: &super::super::TraitHeader,
    (flags, members, supertraits): (u16, usize, usize),
) -> Result<(), &'static str> {
    if header.flags.bits() != flags
        || header.written_parameters != 0
        || header.region_captures != 0
        || header.members.slice(&rows.members)?.len() != members
        || header
            .written_supertraits
            .slice(&rows.clause_indices)?
            .len()
            != supertraits
    {
        return Err("source restricted compiler trait changed its actual declaration profile");
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

fn require_transmute_header(header: &super::super::TraitHeader) -> Result<(), &'static str> {
    let flags = TraitFlags::UNSAFE
        | TraitFlags::DENY_IMPL
        | TraitFlags::COINDUCTIVE
        | TraitFlags::DYN_INCOMPATIBLE;
    if header.flags.bits() != flags
        || header.written_parameters != 2
        || header.region_captures != 0
        || header.written_supertraits.start != header.written_supertraits.end
    {
        return Err("source transmute_trait requires its actual restricted two-parameter trait");
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
                (roles.languages, roles.definition_policies != 0),
                Constness::Ordinary,
                roles.pin,
                work,
            )? || bodies
                && (captured::method(
                    rows,
                    headers,
                    header,
                    (roles.definition_policies != 0, Constness::Ordinary),
                    roles.pin,
                    work,
                )? || reservation_parent(rows, headers, header, work)?.is_some())))
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
    let modern = roles.definition_policies != 0;
    let inherent_const_impl = modern
        && implementation.trait_application.is_none()
        && implementation.polarity.is_positive();
    let owner = if inherent_const_impl {
        None
    } else {
        let Some(owner) = roles.const_trait_impl(rows, headers, implementation, work)? else {
            return Ok(false);
        };
        Some(owner)
    };
    Ok(conversions::method(
        rows,
        headers,
        header,
        (modern && roles.languages, modern),
        Constness::Const,
        roles.pin,
        work,
    )? || (modern || owner.is_some_and(|owner| roles.dereferences.contains(&Some(owner))))
        && captured::method(
            rows,
            headers,
            header,
            (true, Constness::Const),
            roles.pin,
            work,
        )?)
}

fn ordinary_trait_default(
    headers: &HeaderIndex<'_>,
    header: &super::super::FunctionHeader,
    work: &mut Work,
) -> Result<bool, &'static str> {
    let Some(parent) = header.parent.filter(|_| {
        header.constness == Constness::Ordinary && matches!(header.body, FunctionBody::Original(_))
    }) else {
        return Ok(false);
    };
    Ok(matches!(
        headers.header(parent, work)?,
        Some(super::super::HeaderRef::Trait(_))
    ))
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
        .is_some_and(|implementation| {
            implementation.trait_application.is_none() && !implementation.constant
        }))
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
    work.charge(rows.declarations.len())?;
    if profile == NamespaceProfile::Root
        && rows.declarations.iter().any(|declaration| {
            declaration.definition.kind == super::super::DefinitionKind::Module
                && declaration.definition.parent.is_some()
        })
    {
        return Err("nested source modules require a wider source namespace profile");
    }
    if roles.definition_policies == 0 {
        require_simple_formals(rows, headers, work)?;
    }
    for header in &rows.functions {
        work.charge(1 + 2 * rows.exact_callables.len().checked_ilog2().unwrap_or(0) as usize)?;
        captured::require_profile(rows, headers, header, roles.definition_policies != 0, work)?;
        let free_effects = profile.callables() && header.parent.is_none();
        // Version thirteen: an associated function may also be unsafe. Its
        // constness, convention and variadic forms keep their fences.
        let safety = free_effects || (bodies && header.parent.is_some());
        let callable = callable_roles::required_member(header, roles.callables, work)?;
        let provided_default = roles.provided_default(rows, headers, header, work)?;
        let conditional = callable
            || provided_default
            || profile.conversions() && roles.required_member(rows, headers, header, work)?;
        let inherent_const = bodies && inherent_const(rows, headers, header, work)?;
        let trait_const =
            bodies && !inherent_const && trait_const_method(rows, headers, header, roles, work)?;
        let async_callable =
            callable_roles::async_required_member(header, roles.async_callables, work)?;
        let callable_impl = roles.definition_policies != 0
            && callable_roles::implementation_member(
                rows,
                headers,
                header,
                (roles.callables, roles.async_callables),
                work,
            )?;
        let supported_signature = |signature: super::super::Signature| {
            (safety || !signature.unsafe_)
                && !signature.variadic
                && (callable
                    || async_callable
                    || callable_impl
                    || admitted_convention(
                        rows,
                        header.declaration,
                        signature.convention,
                        profile,
                        roles.definition_policies != 0,
                    ))
        };
        if (!free_effects
            && !conditional
            && !inherent_const
            && !trait_const
            && header.constness != Constness::Ordinary)
            || !conditional && header.constness == Constness::Conditional
            || !supported_signature(header.original)
            || !supported_signature(header.callable)
        {
            return Err(
                "source nonordinary callable effect/safety/convention requires a wider profile",
            );
        }
        require_public_callable(header, free_effects, bodies)?;
        let associated = provided_default
            || trait_const
            || (roles.definition_policies != 0 && ordinary_trait_default(headers, header, work)?)
            || associated_method(rows, headers, header, work, profile, roles, bodies)?;
        if matches!(header.body, FunctionBody::Original(_))
            && header.parent.is_some()
            && !associated
        {
            return Err("source provided associated bodies require a wider explicit profile");
        }
        let free_regions = bodies
            && header.parent.is_none()
            && header.regions.captures == 0
            && (roles.definition_policies != 0
                || header.regions.early.start == header.regions.early.end);
        if matches!(header.body, FunctionBody::Original(_))
            && header.regions.total != 0
            && !(profile.conversions() && (associated || free_regions))
        {
            return Err("source original region formals require their complete wider body profile");
        }
        check_body_owner(rows, header, associated, profile, bodies, work)?;
        headers.header(header.declaration, work)?;
    }
    Ok(())
}

fn check_body_owner(
    rows: &SourceRows,
    header: &super::super::FunctionHeader,
    associated: bool,
    profile: NamespaceProfile,
    bodies: bool,
    work: &mut Work,
) -> Result<(), &'static str> {
    let parent = rows.declarations[header.declaration.0 as usize]
        .definition
        .parent
        .ok_or("source function lacks its actual lexical declaration owner")?;
    let parent_kind = rows.declarations[parent.0 as usize].definition.kind;
    match header.body {
        FunctionBody::Absent
            if (header.parent != Some(parent)
                || parent_kind != super::super::DefinitionKind::Trait)
                && !rows.bodyless_runtime(header.declaration, work)?
                && rows.declared_intrinsic(header.declaration, work)?.is_none() =>
        {
            return Err("source body absence is supported only for actual required trait members");
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
    Ok(())
}

fn require_public_callable(
    header: &super::super::FunctionHeader,
    free_effects: bool,
    bodies: bool,
) -> Result<(), &'static str> {
    if free_effects
        && (header.original.unsafe_ != header.callable.unsafe_
            || header.original.convention != header.callable.convention)
    {
        return Err("source free callable changed its actual original/public safety or convention");
    }
    if bodies && header.parent.is_some() && header.original.unsafe_ != header.callable.unsafe_ {
        return Err("source associated callable changed its actual original/public safety");
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
    (binding, bootstrap): (BindingProfile, bool),
    features: bool,
    roles: &Bindings,
) -> Result<(), &'static str> {
    let regional = profile.conversions();
    let constant_initializers = binding == BindingProfile::BodyOperations;
    for original in &rows.originals {
        work.charge(1)?;
        let expected = if profile.callables() {
            callable_traits::original_context(
                rows,
                headers,
                original,
                work,
                (constant_initializers, bootstrap),
                roles.runtime_format_bodies,
                roles.default_traits(),
            )?
        } else {
            super::super::CallContext::Runtime
        };
        if original.context != expected
            || original.precise_live_drops
                && (!constant_initializers || expected == super::super::CallContext::Runtime)
            || !regional && !original.variables.is_empty()
            || !regional && !original.projections.is_empty()
            || !features && (!original.calls.is_empty() || !original.arguments.is_empty())
            || !regional && !original.openings.is_empty()
        {
            return Err("source contextual/borrow/call original requires a wider explicit profile");
        }
        for value in &original.values {
            work.charge(1)?;
            if !original_value(value.kind, features, regional, constant_initializers) {
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
fn defining_default(
    rows: &SourceRows,
    headers: &HeaderIndex<'_>,
    origin: EnvironmentOrigin,
    defaults: callable_traits::DefaultTraits,
    work: &mut Work,
) -> Result<(), &'static str> {
    let EnvironmentOrigin::ConstTraitDefault { member, trait_ } = origin else {
        return Ok(());
    };
    if trait_.artifact == rows.policy.provider
        && let Some(super::super::HeaderRef::Function(index)) = headers.header(member, work)?
    {
        let function = &rows.functions[index as usize];
        if function.parent == Some(trait_.declaration)
            && defaults.provided(rows, headers, function, work)?
        {
            return Ok(());
        }
    }
    Err("source defining default requires its wider imported preparation profile")
}

/// Older profiles carry only vacuous callable clause scopes. Version fourteen
/// retains the complete scoped formation term under its actual clause binder.
fn body_inputs(
    rows: &SourceRows,
    headers: &HeaderIndex<'_>,
    work: &mut Work,
    regional: bool,
    scoped: bool,
    defaults: callable_traits::DefaultTraits,
    declaration_policies: bool,
) -> Result<(), &'static str> {
    for input in &rows.body_inputs {
        work.charge(1)?;
        defining_default(rows, headers, input.environment, defaults, work)?;
        if let Some(builtins) = &input.builtins {
            defining_default(rows, headers, builtins.environment, defaults, work)?;
            for request in &builtins.predicates {
                work.charge(1)?;
                if request.binder.is_some() {
                    return Err("source builtin requirement cannot carry a clause quantifier");
                }
                ordinary(&request.value, declaration_policies, declaration_policies)?;
            }
        }
        for request in &input.type_assumptions {
            work.charge(1)?;
            if request.binder.is_some() && !declaration_policies {
                return Err(
                    "quantified original assumptions require the declaration policy profile",
                );
            }
            ordinary(&request.value, false, false)?;
        }
        normalization_ownership(input, work, regional, declaration_policies)?;
        for obligation in &input.obligations {
            work.charge(1)?;
            let supported_scope = scoped
                && matches!(obligation.kind, InputKind::ScopedWellFormed { frame, .. }
                    if rows.binders.get(frame.0 as usize).is_some_and(|b|
                        b.count == 0 || declaration_policies));
            let supported_membership =
                declaration_policies && matches!(obligation.kind, InputKind::Membership { .. });
            if !supported_scope
                && !supported_membership
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
    declaration_policies: bool,
) -> Result<(), &'static str> {
    for normalization in &input.normalizations {
        work.charge(1)?;
        if normalization.scoped.is_some() && !declaration_policies {
            return Err("source scoped normalization requires the declaration policy profile");
        }
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
/// `conditional` admits `[const]` bounds; `unconditional` admits v14 `const` bounds.
fn ordinary(
    request: &super::super::Requirement,
    conditional: bool,
    unconditional: bool,
) -> Result<(), &'static str> {
    if matches!(request, super::super::Requirement::Outlives(_)) {
        return Err("standalone regional clauses require their explicit wider header consumer");
    }
    let allowed = match request {
        super::super::Requirement::Trait { effect, .. } => match effect {
            Constness::Ordinary => true,
            Constness::Conditional => conditional,
            Constness::Const => unconditional,
        },
        _ => true,
    };
    if !allowed {
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
    unsize: Option<crate::support::crates::owner_catalog::DeclarationRef>,
    pointee: Option<crate::support::crates::owner_catalog::DeclarationRef>,
    transmute: Option<crate::support::crates::owner_catalog::DeclarationRef>,
    discriminant: Option<crate::support::crates::owner_catalog::DeclarationRef>,
    fn_pointer: Option<crate::support::crates::owner_catalog::DeclarationRef>,
    field: Option<crate::support::crates::owner_catalog::DeclarationRef>,
    drop: Option<crate::support::crates::owner_catalog::DeclarationRef>,
    runtime_format_bodies: [Option<crate::support::crates::owner_catalog::DeclarationRef>;
        LangItem::RUNTIME_FORMAT_BODIES.len()],
    drop_glue: Option<crate::support::crates::owner_catalog::DeclarationRef>,
    clone: Option<crate::support::crates::owner_catalog::DeclarationRef>,
    clone_fn: Option<crate::support::crates::owner_catalog::DeclarationRef>,
    clone_from: Option<crate::support::crates::owner_catalog::DeclarationRef>,
    iterator_defaults: [Option<crate::support::crates::owner_catalog::DeclarationRef>;
        LangItem::PROVIDED_ITERATOR_METHODS.len()],
    trivial_clone: Option<crate::support::crates::owner_catalog::DeclarationRef>,
    try_methods: [Option<crate::support::crates::owner_catalog::DeclarationRef>;
        LangItem::REQUIRED_TRY_METHODS.len()],
    try_traits: [Option<crate::support::crates::owner_catalog::DeclarationRef>; 2],
    for_methods: [Option<crate::support::crates::owner_catalog::DeclarationRef>;
        LangItem::REQUIRED_FOR_METHODS.len()],
    for_traits: [Option<crate::support::crates::owner_catalog::DeclarationRef>; 2],
    comparisons: [Option<crate::support::crates::owner_catalog::DeclarationRef>;
        LangItem::COMPARISON_TRAITS.len()],
    pin: Option<crate::support::crates::owner_catalog::DeclarationRef>,
    conversion: Option<crate::support::crates::owner_catalog::DeclarationRef>,
    binary_operators: [Option<crate::support::crates::owner_catalog::DeclarationRef>;
        Profile::CONST_BINARY_ROLES.len()],
    dereferences: [Option<crate::support::crates::owner_catalog::DeclarationRef>;
        Profile::CONST_DEREF_ROLES.len()],
    range: Option<interval_roles::Family>,
    callables: callable_roles::Roles,
    async_callables: [Option<crate::support::crates::owner_catalog::DeclarationRef>; 3],
    /// Version thirteen: exact empty auto declarations, unsafe traits and
    /// every trait impl's own safety and polarity.
    languages: bool,
    definition_policies: u16,
    /// The role-bound auto markers, in `callable_traits::MARKERS` order.
    markers: [Option<crate::support::crates::owner_catalog::DeclarationRef>; 4],
}
impl Bindings {
    fn restricted_trait(
        &self,
        declaration: crate::support::crates::owner_catalog::DeclarationRef,
        flags: u16,
    ) -> Option<(u16, usize, usize)> {
        let restricted = TraitFlags::DENY_IMPL | TraitFlags::DYN_INCOMPATIBLE;
        if self.fn_pointer == Some(declaration) {
            Some((restricted, 1, 2))
        } else if self.field == Some(declaration) {
            Some((restricted | TraitFlags::UNSAFE, 3, 3))
        } else if self.pointee == Some(declaration) && flags & TraitFlags::DENY_IMPL != 0 {
            Some((restricted, 1, 1))
        } else {
            None
        }
    }

    fn default_traits(&self) -> callable_traits::DefaultTraits {
        callable_traits::DefaultTraits {
            drop: self.drop,
            clone_from: self.clone_from,
            iterator_defaults: self.iterator_defaults,
            comparisons: self.comparisons,
            general: self.definition_policies != 0,
        }
    }

    fn const_operator(
        &self,
        declaration: crate::support::crates::owner_catalog::DeclarationRef,
    ) -> bool {
        self.binary_operators.contains(&Some(declaration))
            || self.dereferences.contains(&Some(declaration))
            || self.comparisons.contains(&Some(declaration))
    }

    fn const_trait_impl(
        &self,
        rows: &SourceRows,
        headers: &HeaderIndex<'_>,
        implementation: &super::super::ImplHeader,
        work: &mut Work,
    ) -> Result<Option<crate::support::crates::owner_catalog::DeclarationRef>, &'static str> {
        let Some(application) = implementation.trait_application else {
            return Ok(None);
        };
        let modern = self.definition_policies != 0;
        if !implementation.constant
            || (implementation.unsafe_ && !(modern && self.languages))
            || !implementation.polarity.is_positive()
            || (!modern
                && self.for_traits[1].is_none()
                && self
                    .binary_operators
                    .iter()
                    .chain(&self.dereferences)
                    .chain(&self.comparisons)
                    .all(Option::is_none))
        {
            return Ok(None);
        }
        work.charge(1)?;
        let declaration = rows
            .trait_applications
            .get(application.0 as usize)
            .ok_or("source const operator impl lost its actual trait application")?
            .declaration;
        if declaration.artifact.0 != 0 {
            return Ok(None);
        }
        if !modern
            && let Some(iteration) = self.iterator_blanket()
            && declaration.declaration == iteration.into_trait
        {
            return iteration
                .check(rows, headers, implementation, work)
                .map(|valid| valid.then_some(declaration.declaration));
        }
        if !modern && !self.const_operator(declaration.declaration) {
            return Ok(None);
        }
        let Some(super::super::HeaderRef::Trait(index)) =
            headers.header(declaration.declaration, work)?
        else {
            return Err("source const operator impl lost its actual trait header");
        };
        Ok(
            (rows.traits[index as usize].flags.bits() & TraitFlags::CONST != 0)
                .then_some(declaration.declaration),
        )
    }

    fn iterator_blanket(&self) -> Option<iterator_blanket::Contract> {
        Some(iterator_blanket::Contract {
            iterator: self.for_traits[0]?,
            meta_sized: self.sizing[1]?,
            into_trait: self.for_traits[1]?,
            into_method: self.for_methods[1]?,
        })
    }

    fn conditional_iterator_input(
        &self,
        rows: &SourceRows,
        headers: &HeaderIndex<'_>,
        input: &super::super::Clause,
        work: &mut Work,
    ) -> Result<bool, &'static str> {
        let Some(iteration) = self.iterator_blanket() else {
            return Ok(false);
        };
        let Some(subject) = iteration.premise(rows, input) else {
            return Ok(false);
        };
        let Some(super::super::HeaderRef::Implementation(index)) =
            headers.header(input.owner, work)?
        else {
            return Ok(false);
        };
        let implementation = &rows.implementations[index as usize];
        if subject != implementation.self_type {
            return Ok(false);
        }
        iteration.check(rows, headers, implementation, work)
    }

    fn conditional_role_input(&self, rows: &SourceRows, input: &super::super::Clause) -> bool {
        let [deref, deref_mut] = self.dereferences;
        let [try_, from_residual] = self.try_traits;
        let [partial_eq, partial_ord] = self.comparisons;
        let [_, _, fold, last, count] = self.iterator_defaults;
        let [_, _, fn_mut, _] = self.callables.0;
        let nested_reduction = rows
            .declarations
            .get(input.owner.0 as usize)
            .and_then(|row| row.definition.parent)
            .is_some_and(|parent| [last, count].contains(&Some(parent)));
        let target = if [self.drop_glue, self.clone_from, fold, last, count]
            .contains(&Some(input.owner))
            || nested_reduction
        {
            self.destruct
        } else if Some(input.owner) == self.trivial_clone {
            self.clone
        } else if Some(input.owner) == deref_mut {
            deref
        } else if Some(input.owner) == try_ {
            from_residual
        } else if Some(input.owner) == partial_ord {
            partial_eq
        } else {
            return false;
        };
        let super::super::Requirement::Trait {
            application,
            effect: Constness::Conditional,
        } = input.requirement
        else {
            return false;
        };
        rows.trait_applications
            .get(application.0 as usize)
            .is_some_and(|app| {
                let declared = Some(app.declaration.declaration);
                app.declaration.artifact.0 == 0
                    && (declared == target || Some(input.owner) == fold && declared == fn_mut)
            })
    }

    fn provided_default(
        &self,
        rows: &SourceRows,
        headers: &HeaderIndex<'_>,
        function: &super::super::FunctionHeader,
        work: &mut Work,
    ) -> Result<bool, &'static str> {
        if !self
            .default_traits()
            .provided(rows, headers, function, work)?
        {
            return Ok(false);
        }
        let Some(owner) = function.parent else {
            return Ok(false);
        };
        if !matches!(headers.header(owner, work)?, Some(super::super::HeaderRef::Trait(index))
            if rows.traits[index as usize].flags.bits() & TraitFlags::CONST != 0)
        {
            return Err("source provided default requires its actual const trait");
        }
        Ok(true)
    }

    fn required_member(
        &self,
        rows: &SourceRows,
        headers: &HeaderIndex<'_>,
        function: &super::super::FunctionHeader,
        work: &mut Work,
    ) -> Result<bool, &'static str> {
        if self.drop.is_some() && function.parent == self.drop
            || self.clone_fn == Some(function.declaration)
            || self.try_methods.contains(&Some(function.declaration))
            || self.for_methods.contains(&Some(function.declaration))
        {
            return conversions::required_member(
                rows,
                headers,
                function,
                function.parent,
                true,
                work,
            );
        }
        if self
            .range
            .is_some_and(|family| family.method == function.declaration)
        {
            return Ok(true);
        }
        let operator = function.parent.filter(|&owner| self.const_operator(owner));
        if conversions::required_member(
            rows,
            headers,
            function,
            operator.or(self.conversion),
            operator.is_some(),
            work,
        )? {
            return Ok(true);
        }
        if self.definition_policies == 0
            || function.body != FunctionBody::Absent
            || function.constness != Constness::Conditional
        {
            return Ok(false);
        }
        let Some(owner) = function.parent else {
            return Ok(false);
        };
        Ok(matches!(
            headers.header(owner, work)?,
            Some(super::super::HeaderRef::Trait(index))
                if rows.traits[index as usize].flags.bits() & TraitFlags::CONST != 0
        ))
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
fn body_operation_role(role: LangItem) -> bool {
    matches!(
        role,
        LangItem::TypeInfo
            | LangItem::TransmuteOpts
            | LangItem::TransmuteTrait
            | LangItem::FnPtrTrait
            | LangItem::CStr
            | LangItem::Context
            | LangItem::ResumeTy
            | LangItem::FnPtrAddr
            | LangItem::AllocLayout
            | LangItem::TypeId
            | LangItem::PointeeTrait
            | LangItem::MetadataType
            | LangItem::DiscriminantKind
            | LangItem::DiscriminantType
            | LangItem::TrivialClone
            | LangItem::DynMetadata
            | LangItem::ConstParamTy
            | LangItem::StructuralPartialEq
            | LangItem::PanicLocation
            | LangItem::PanicInfo
            | LangItem::FormatArguments
            | LangItem::FormatArgument
            | LangItem::MemSizeConst
            | LangItem::MemAlignConst
            | LangItem::DropGlue
            | LangItem::AsyncDrop
            | LangItem::PinNewUnchecked
            | LangItem::DerefTarget
            | LangItem::ReceiverTarget
            | LangItem::SliceLen
            | LangItem::OffsetOf
            | LangItem::RangeInclusiveNew
            | LangItem::BikeshedGuaranteedNoDrop
    ) || interval_roles::ROLES.contains(&role)
        || LangItem::RANGE_NOMINALS.contains(&role)
        || LangItem::NOMINAL_WRAPPERS.contains(&role)
        || LangItem::GUARD_PANICS.contains(&role)
        || LangItem::FORMAT_PANICS.contains(&role)
        || LangItem::SOURCE_HELPERS.contains(&role)
        || LangItem::RESUMABLE_REQUIRED.contains(&role)
        || LangItem::ASYNC_GENERATOR_HELPERS.contains(&role)
        || LangItem::FIELD_REFLECTION.contains(&role)
        || role.resumable_associated()
        || LangItem::TRY_PROTOCOL.contains(&role)
        || LangItem::FOR_PROTOCOL.contains(&role)
        || LangItem::COMPARISON_TRAITS.contains(&role)
        || role.is_enum_value_role()
        || role.is_runtime_import()
}
fn role_allowed(profile: BindingProfile, role: LangItem) -> bool {
    if body_operation_role(role) {
        return profile == BindingProfile::BodyOperations;
    }
    if matches!(
        role,
        LangItem::UnwindSafe
            | LangItem::RefUnwindSafe
            | LangItem::Start
            | LangItem::Termination
            | LangItem::BeginPanic
            | LangItem::EhPersonality
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
        let address = role.declaration;
        if address.artifact != super::super::ArtifactRef(0)
            || declarations.contains(&Some(address.declaration))
            || declarations[role.role.index()].is_some()
        {
            return Err("source role binding is duplicated or belongs to a foreign provider");
        }
        let header = headers.header(address.declaration, work)?;
        let declaration = rows
            .declarations
            .get(address.declaration.0 as usize)
            .ok_or("source role binding names an unknown declaration")?;
        role_header(
            profile,
            rows,
            declaration.definition.kind,
            role.role,
            header,
        )?;
        if !role_allowed(profile, role.role) {
            return Err("source role requires a wider explicit source profile");
        }
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
    bindings_with_policy(rows, headers, work, profile, false)
}
fn bindings_with_policy(
    rows: &SourceRows,
    headers: &HeaderIndex<'_>,
    work: &mut Work,
    profile: BindingProfile,
    bootstrap: bool,
) -> Result<Bindings, &'static str> {
    let mut declarations = binding_declarations(rows, headers, work, profile)?;
    diagnostic_bindings(rows, headers, work, profile, &mut declarations)?;
    owned_role_members(rows, headers, work, &declarations)?;
    let bound = |role: LangItem| declarations[role.index()];
    let sizing = [LangItem::Sized, LangItem::MetaSized, LangItem::PointeeSized].map(bound);
    if !bootstrap && sizing.contains(&None) {
        return Err("source functions require the real complete sizing trio");
    }
    slot_bindings(rows, headers, work, profile, sizing[0], bootstrap)?;
    let languages = profile.languages();
    let callables = callable_roles::Roles(
        [
            LangItem::Tuple,
            LangItem::FnOnce,
            LangItem::FnMut,
            LangItem::Fn,
        ]
        .map(|role| bound(role).filter(|_| languages)),
    );
    let modern = profile == BindingProfile::BodyOperations;
    let body_role = |role| bound(role).filter(|_| modern);
    let range = if modern {
        interval_roles::family(rows, headers, interval_roles::ROLES.map(bound), work)?
    } else {
        None
    };
    let marker_traits = matches!(
        profile,
        BindingProfile::Callables
            | BindingProfile::Conversions
            | BindingProfile::Languages
            | BindingProfile::BodyOperations
    );
    let [clone_from] = const_default_members(
        rows,
        headers,
        body_role(LangItem::Clone),
        ["clone_from"],
        work,
    )?;
    let iterator_defaults = const_default_members(
        rows,
        headers,
        body_role(LangItem::Iterator),
        LangItem::PROVIDED_ITERATOR_METHODS,
        work,
    )?;
    Ok(Bindings {
        callables,
        async_callables: LangItem::ASYNC_CALL_TRAITS.map(body_role),
        languages,
        range,
        drop: body_role(LangItem::Drop),
        drop_glue: bound(LangItem::DropGlue),
        clone: body_role(LangItem::Clone),
        clone_fn: body_role(LangItem::CloneFn),
        clone_from,
        iterator_defaults,
        trivial_clone: bound(LangItem::TrivialClone),
        try_methods: LangItem::REQUIRED_TRY_METHODS.map(bound),
        try_traits: [
            bound(LangItem::TryTrait),
            required_method_owner(rows, headers, bound(LangItem::TryFromResidual), work)?,
        ],
        for_methods: LangItem::REQUIRED_FOR_METHODS.map(bound),
        for_traits: for_trait_owners(rows, headers, work, &declarations)?,
        runtime_format_bodies: LangItem::RUNTIME_FORMAT_BODIES.map(bound),
        comparisons: LangItem::COMPARISON_TRAITS.map(bound),
        pin: bound(LangItem::Pin),
        definition_policies: profile.definition_policies(),
        markers: [
            LangItem::Send,
            LangItem::Sync,
            LangItem::Unpin,
            LangItem::UnsafeUnpin,
        ]
        .map(|role| bound(role).filter(|_| languages)),
        conversion: (matches!(
            profile,
            BindingProfile::Conversions
                | BindingProfile::CompilerOperations
                | BindingProfile::Languages
                | BindingProfile::BodyOperations
        ))
        .then_some(bound(LangItem::From))
        .flatten(),
        binary_operators: Profile::CONST_BINARY_ROLES.map(body_role),
        dereferences: Profile::CONST_DEREF_ROLES.map(body_role),
        sizing,
        destruct: marker_traits.then_some(bound(LangItem::Destruct)).flatten(),
        no_drop: bound(LangItem::BikeshedGuaranteedNoDrop),
        unsize: body_role(LangItem::Unsize),
        pointee: body_role(LangItem::PointeeTrait),
        transmute: body_role(LangItem::TransmuteTrait),
        discriminant: bound(LangItem::DiscriminantKind),
        fn_pointer: bound(LangItem::FnPtrTrait),
        field: bound(LangItem::Field),
        freeze: marker_traits.then_some(bound(LangItem::Freeze)).flatten(),
    })
}

/// Each diagnostic identity names one of the provider's own declarations.
/// Diagnostic-owned marker slots use the same declaration index as language roles.
fn diagnostic_bindings(
    rows: &SourceRows,
    headers: &HeaderIndex<'_>,
    work: &mut Work,
    profile: BindingProfile,
    declarations: &mut [Option<crate::support::crates::owner_catalog::DeclarationRef>],
) -> Result<(), &'static str> {
    let diagnostics = rows.diagnostics();
    if !diagnostics.is_empty() && !profile.languages() {
        return Err("source diagnostic identity requires a language role table");
    }
    let mut seen = [false; super::super::DiagnosticRole::ALL.len()];
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
            super::super::DiagnosticRole::Send
            | super::super::DiagnosticRole::UnwindSafe
            | super::super::DiagnosticRole::RefUnwindSafe => {
                let marker = row
                    .role
                    .auto_marker()
                    .ok_or("source diagnostic marker disappeared")?;
                validate_diagnostic_marker(rows, profile, marker, kind, header)?;
                work.charge(declarations.len())?;
                if declarations.contains(&Some(row.declaration.declaration)) {
                    return Err("source diagnostic marker shares another role's declaration");
                }
                declarations[marker.index()] = Some(row.declaration.declaration);
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
            super::super::DiagnosticRole::Display => {
                work.charge(declarations.len())?;
                if profile != BindingProfile::BodyOperations
                    || kind != Some(super::super::DefinitionKind::Trait)
                    || !matches!(header, Some(super::super::HeaderRef::Trait(_)))
                    || declarations.contains(&Some(row.declaration.declaration))
                {
                    return Err(
                        "source Display diagnostic identity requires its own version-fourteen trait",
                    );
                }
            }
            super::super::DiagnosticRole::Result => {
                result_diagnostic(
                    rows,
                    row.declaration.declaration,
                    declarations,
                    profile,
                    (kind, header),
                    work,
                )?;
            }
        }
    }
    diagnostic_role_requirements(seen, declarations)
}

fn validate_diagnostic_marker(
    rows: &SourceRows,
    profile: BindingProfile,
    role: LangItem,
    kind: Option<super::super::DefinitionKind>,
    header: Option<super::super::HeaderRef>,
) -> Result<(), &'static str> {
    let (flags, error) = if role == LangItem::Send {
        (TraitFlags::UNSAFE | TraitFlags::AUTO, SEND)
    } else {
        if profile != BindingProfile::BodyOperations {
            return Err("source unwind safety diagnostic identities require version fourteen");
        }
        (
            TraitFlags::AUTO,
            "source unwind safety diagnostic item requires its safe auto trait declaration",
        )
    };
    let Some(super::super::HeaderRef::Trait(index)) = header else {
        return Err(error);
    };
    if kind != Some(super::super::DefinitionKind::Trait)
        || rows
            .traits
            .get(index as usize)
            .map(|row| row.flags.bits() & !profile.definition_policies())
            != Some(flags)
    {
        return Err(error);
    }
    Ok(())
}

fn diagnostic_role_requirements(
    seen: [bool; super::super::DiagnosticRole::ALL.len()],
    declarations: &[Option<crate::support::crates::owner_catalog::DeclarationRef>],
) -> Result<(), &'static str> {
    let const_marker = declarations[LangItem::ConstParamTy.index()].is_some();
    if seen[super::super::DiagnosticRole::Eq as usize] != const_marker
        || const_marker && declarations[LangItem::StructuralPartialEq.index()].is_none()
    {
        return Err(
            "source ConstParamTy requires its actual StructuralPartialEq and Eq declarations",
        );
    }
    if seen[super::super::DiagnosticRole::Display as usize]
        != declarations[LangItem::PanicDisplay.index()].is_some()
    {
        return Err("source panic_display requires its actual Display diagnostic declaration");
    }
    for role in [LangItem::ResultOk, LangItem::ResultErr] {
        if seen[super::super::DiagnosticRole::Result as usize]
            != declarations[role.index()].is_some()
        {
            return Err("source Result variants require their actual diagnostic enum declaration");
        }
    }
    Ok(())
}

fn result_diagnostic(
    rows: &SourceRows,
    result: crate::support::crates::owner_catalog::DeclarationRef,
    declarations: &[Option<crate::support::crates::owner_catalog::DeclarationRef>],
    profile: BindingProfile,
    (kind, header): (
        Option<super::super::DefinitionKind>,
        Option<super::super::HeaderRef>,
    ),
    work: &mut Work,
) -> Result<(), &'static str> {
    if profile != BindingProfile::BodyOperations
        || kind != Some(super::super::DefinitionKind::Enum)
        || !matches!(header, Some(super::super::HeaderRef::Nominal(_)))
    {
        return Err("source Result diagnostic identity requires its own version-fourteen enum");
    }
    for role in [LangItem::ResultOk, LangItem::ResultErr] {
        work.charge(1)?;
        let actual = declarations[role.index()]
            .and_then(|variant| rows.declarations.get(variant.0 as usize))
            .and_then(|row| row.definition.parent);
        if actual != Some(result) {
            return Err("source Result variant belongs to another actual diagnostic enum");
        }
    }
    Ok(())
}

const SEND: &str = "imported Send diagnostic item requires its unsafe auto trait declaration";
fn defined_operation_role(
    profile: BindingProfile,
    rows: &SourceRows,
    role: LangItem,
    header: Option<super::super::HeaderRef>,
) -> Result<(), &'static str> {
    if profile == BindingProfile::BodyOperations
        && let Some(super::super::HeaderRef::Function(index)) = header
        && rows.functions.get(index as usize).is_some_and(|function| {
            function.parent.is_none()
                && matches!(function.body, FunctionBody::Original(_))
                && (function.constness == Constness::Const
                    || role != LangItem::DropGlue && !LangItem::FORMAT_PANICS.contains(&role))
        })
    {
        return Ok(());
    }
    Err(if role == LangItem::DropGlue {
        "source drop_glue requires its actual defined const module function"
    } else if LangItem::FORMAT_PANICS.contains(&role) {
        "source panic role requires its actual defined const module function"
    } else if LangItem::SOURCE_HELPERS.contains(&role) {
        "source helper role requires its actual defined free function"
    } else {
        "source guard role requires its actual defined free function"
    })
}

/// The declaration/header kind each bound role may claim under `profile`.
fn role_header(
    profile: BindingProfile,
    rows: &SourceRows,
    kind: super::super::DefinitionKind,
    role: LangItem,
    header: Option<super::super::HeaderRef>,
) -> Result<(), &'static str> {
    use super::super::HeaderRef;
    let modern = profile == BindingProfile::BodyOperations;
    match (role, header) {
        (role, header)
            if LangItem::TRY_PROTOCOL.contains(&role) || LangItem::FOR_PROTOCOL.contains(&role) =>
        {
            protocol_role_header(rows, role, header)?;
        }
        (role, header) if role.is_enum_value_role() => enum_role_header(kind, role, header)?,
        (role, header)
            if role == LangItem::DropGlue
                || LangItem::GUARD_PANICS.contains(&role)
                || LangItem::FORMAT_PANICS.contains(&role)
                || LangItem::SOURCE_HELPERS.contains(&role) =>
        {
            defined_operation_role(profile, rows, role, header)?;
        }
        (role, Some(HeaderRef::Function(index)))
            if (role == LangItem::FnPtrAddr || LangItem::RESUMABLE_REQUIRED.contains(&role))
                && required_ordinary_header(rows, index) => {}
        (role, Some(HeaderRef::AssociatedType(_)))
            if role.resumable_associated()
                || matches!(role, LangItem::FieldBase | LangItem::FieldType) => {}
        (role, header) if LangItem::ASYNC_GENERATOR_HELPERS.contains(&role) => {
            async_generator_header(rows, role, header)?;
        }
        (LangItem::OffsetOf, Some(HeaderRef::Function(index)))
            if offset_intrinsic_header(rows, index) => {}
        (role, header)
            if role == LangItem::RangeInclusiveNew || LangItem::RANGE_NOMINALS.contains(&role) =>
        {
            range_value_header(profile, rows, kind, role, header)?;
        }
        (role, Some(HeaderRef::Function(_))) if role.is_runtime_import() && modern => {}
        (role, Some(HeaderRef::Nominal(index)))
            if (role == LangItem::AllocLayout || LangItem::NOMINAL_WRAPPERS.contains(&role))
                && modern
                && wrapper_header(rows, kind, role, index) => {}
        (role, _) if LangItem::NOMINAL_WRAPPERS.contains(&role) && modern => {
            return Err("source wrapper role requires its actual nominal header");
        }
        (LangItem::PointeeTrait | LangItem::DiscriminantKind, Some(HeaderRef::Trait(_)))
        | (
            LangItem::MetadataType
            | LangItem::DiscriminantType
            | LangItem::DerefTarget
            | LangItem::ReceiverTarget,
            Some(HeaderRef::AssociatedType(_)),
        )
        | (
            LangItem::RangeMin
            | LangItem::RangeMax
            | LangItem::MemSizeConst
            | LangItem::MemAlignConst
            | LangItem::FieldOffset,
            Some(HeaderRef::Constant(_)),
        )
        | (
            LangItem::RangeSub | LangItem::PinNewUnchecked | LangItem::SliceLen,
            Some(HeaderRef::Function(_)),
        )
        | (
            LangItem::DynMetadata | LangItem::TypeInfo | LangItem::TypeId | LangItem::PanicLocation,
            Some(HeaderRef::Nominal(_)),
        ) if profile == BindingProfile::BodyOperations => {}
        (LangItem::PointeeTrait | LangItem::MetadataType | LangItem::DynMetadata, _) => {
            return Err("source Pointee roles require their actual declaration header kinds");
        }
        (LangItem::DerefTarget | LangItem::PinNewUnchecked, _) => {
            return Err("source pinned dereference roles require their actual declaration kinds");
        }
        (LangItem::RangeMin | LangItem::RangeMax | LangItem::RangeSub, _) => {
            return Err("source range roles require their actual associated header kinds");
        }
        (LangItem::TypeInfo | LangItem::TypeId, _) => {
            return Err("source reflection role requires its actual struct header");
        }
        (role, header) if ordinary_struct_role(role) => {
            check_struct_role(profile, kind, role, header)?;
        }
        (LangItem::PanicLocation, _) => {
            return Err("source caller location role requires its actual nominal header");
        }
        (
            LangItem::CloneFn | LangItem::FuturePoll | LangItem::AsyncDropInPlace,
            Some(HeaderRef::Function(_)),
        ) => {}
        (LangItem::FutureOutput, Some(HeaderRef::AssociatedType(_)))
            if matches!(
                profile,
                BindingProfile::CompilerOperations | BindingProfile::BodyOperations
            ) => {}
        // A bound `panic` names a free function; consumers re-prove its header.
        // A bound `phantom_data` names a nominal; consumers re-prove its unit struct.
        (LangItem::FnOnceOutput, Some(HeaderRef::AssociatedType(_)))
        | (LangItem::Panic, Some(HeaderRef::Function(_)))
        | (LangItem::PhantomData, Some(HeaderRef::Nominal(_)))
            if profile.languages() => {}
        _ => ordinary_declaration_role(role, header)?,
    }
    Ok(())
}

fn required_ordinary_header(rows: &SourceRows, index: u32) -> bool {
    rows.functions.get(index as usize).is_some_and(|function| {
        function.body == FunctionBody::Absent
            && function.constness == Constness::Ordinary
            && function.parent.is_some()
    })
}

fn async_generator_header(
    rows: &SourceRows,
    role: LangItem,
    header: Option<super::super::HeaderRef>,
) -> Result<(), &'static str> {
    use super::super::HeaderRef;
    let supported = match (role, header) {
        (LangItem::AsyncGenReady, Some(HeaderRef::Function(index))) => {
            rows.functions.get(index as usize).is_some_and(|function| {
                matches!(function.body, FunctionBody::Original(_)) && function.parent.is_some()
            })
        }
        (LangItem::AsyncGenPending | LangItem::AsyncGenFinished, Some(HeaderRef::Constant(_))) => {
            true
        }
        _ => false,
    };
    if !supported {
        return Err("source async generator helper changed its actual function or constant header");
    }
    Ok(())
}

fn ordinary_struct_role(role: LangItem) -> bool {
    matches!(
        role,
        LangItem::PanicInfo
            | LangItem::FormatArguments
            | LangItem::FormatArgument
            | LangItem::TransmuteOpts
            | LangItem::FieldRepresentingType
            | LangItem::CStr
            | LangItem::Context
            | LangItem::ResumeTy
    )
}

fn check_struct_role(
    profile: BindingProfile,
    kind: super::super::DefinitionKind,
    role: LangItem,
    header: Option<super::super::HeaderRef>,
) -> Result<(), &'static str> {
    if profile == BindingProfile::BodyOperations
        && kind == super::super::DefinitionKind::Struct
        && matches!(header, Some(super::super::HeaderRef::Nominal(_)))
    {
        return Ok(());
    }
    Err(match role {
        LangItem::TransmuteOpts => "source transmute options require their actual struct header",
        LangItem::CStr => "source CStr requires its actual struct header",
        _ => "source panic or formatting role requires its actual struct header",
    })
}

fn offset_intrinsic_header(rows: &SourceRows, index: u32) -> bool {
    rows.functions.get(index as usize).is_some_and(|function| {
        function.body == FunctionBody::Intrinsic(super::super::BodylessIntrinsic::OffsetOf)
    })
}

fn range_value_header(
    profile: BindingProfile,
    rows: &SourceRows,
    kind: super::super::DefinitionKind,
    role: LangItem,
    header: Option<super::super::HeaderRef>,
) -> Result<(), &'static str> {
    use super::super::{DefinitionKind, HeaderRef};
    let shared_profile = profile == BindingProfile::BodyOperations;
    let actual = match (role, header) {
        (LangItem::RangeInclusiveNew, Some(HeaderRef::Function(index))) => {
            rows.functions.get(index as usize).is_some_and(|function| {
                function.parent.is_some() && matches!(function.body, FunctionBody::Original(_))
            })
        }
        (role, Some(HeaderRef::Nominal(_))) if LangItem::RANGE_NOMINALS.contains(&role) => {
            kind == DefinitionKind::Struct
        }
        _ => false,
    };
    if !shared_profile || !actual {
        return Err("source range role changed its actual declaration header");
    }
    Ok(())
}

fn ordinary_declaration_role(
    role: LangItem,
    header: Option<super::super::HeaderRef>,
) -> Result<(), &'static str> {
    use super::super::HeaderRef;
    match (role, header) {
        (_, Some(HeaderRef::AssociatedType(_) | HeaderRef::AssociatedDefinition(_))) => {
            return Err("source associated role has a different actual member kind");
        }
        (LangItem::CloneFn, _)
        | (
            _,
            None
            | Some(
                HeaderRef::Function(_)
                | HeaderRef::Implementation(_)
                | HeaderRef::Static(_)
                | HeaderRef::TypeAlias(_)
                | HeaderRef::TraitAlias(_)
                | HeaderRef::Constant(_),
            ),
        ) => return Err("source role has a different actual declaration/header kind"),
        (_, Some(HeaderRef::Trait(_))) => {}
        (_, Some(HeaderRef::Nominal(_) | HeaderRef::Variant(_) | HeaderRef::Constructor(_))) => {
            return Err("ordinary source roles cannot claim a nominal declaration");
        }
    }
    Ok(())
}
fn enum_role_header(
    kind: super::super::DefinitionKind,
    role: LangItem,
    header: Option<super::super::HeaderRef>,
) -> Result<(), &'static str> {
    use super::super::{DefinitionKind, HeaderRef};
    let exact = match role {
        LangItem::Option | LangItem::Ordering | LangItem::Poll | LangItem::CoroutineState => {
            kind == DefinitionKind::Enum && matches!(header, Some(HeaderRef::Nominal(_)))
        }
        _ => kind == DefinitionKind::Variant && matches!(header, Some(HeaderRef::Variant(_))),
    };
    if !exact {
        return Err("source enum role requires its actual enum or variant header");
    }
    Ok(())
}

fn const_default_members<const N: usize>(
    rows: &SourceRows,
    headers: &HeaderIndex<'_>,
    owner: Option<crate::support::crates::owner_catalog::DeclarationRef>,
    names: [&str; N],
    work: &mut Work,
) -> Result<[Option<crate::support::crates::owner_catalog::DeclarationRef>; N], &'static str> {
    use super::super::HeaderRef;
    let mut defaults = [None; N];
    let Some(owner) = owner else {
        return Ok(defaults);
    };
    let Some(HeaderRef::Trait(index)) = headers.header(owner, work)? else {
        return Err("source const default lost its actual trait header");
    };
    let header = &rows.traits[index as usize];
    if header.flags.bits() & TraitFlags::CONST == 0 {
        return Ok(defaults);
    }
    let members = header.members.slice(&rows.members)?;
    work.charge(members.len())?;
    for &member in members {
        let Some(name) = rows
            .declarations
            .get(member.0 as usize)
            .and_then(|row| row.name.as_deref())
        else {
            continue;
        };
        let Some(slot) = names.iter().position(|&expected| expected == name) else {
            continue;
        };
        if !matches!(headers.header(member, work)?, Some(HeaderRef::Function(index))
            if rows.functions[index as usize].parent == Some(owner)
                && rows.functions[index as usize].constness == Constness::Conditional
                && matches!(rows.functions[index as usize].body, FunctionBody::Original(_)))
        {
            return Err("source const default requires its actual provided method");
        }
        if defaults[slot].replace(member).is_some() {
            return Err("source const default repeats an actual provided method");
        }
    }
    Ok(defaults)
}

fn required_method_owner(
    rows: &SourceRows,
    headers: &HeaderIndex<'_>,
    declaration: Option<crate::support::crates::owner_catalog::DeclarationRef>,
    work: &mut Work,
) -> Result<Option<crate::support::crates::owner_catalog::DeclarationRef>, &'static str> {
    let Some(declaration) = declaration else {
        return Ok(None);
    };
    let Some(super::super::HeaderRef::Function(index)) = headers.header(declaration, work)? else {
        return Err("source required method lost its actual header");
    };
    rows.functions[index as usize]
        .parent
        .map(Some)
        .ok_or("source required method lost its actual trait owner")
}

fn for_trait_owners(
    rows: &SourceRows,
    headers: &HeaderIndex<'_>,
    work: &mut Work,
    roles: &[Option<crate::support::crates::owner_catalog::DeclarationRef>; LangItem::ALL.len()],
) -> Result<[Option<crate::support::crates::owner_catalog::DeclarationRef>; 2], &'static str> {
    let [iterator, next, into_iter] = LangItem::FOR_PROTOCOL.map(|role| roles[role.index()]);
    if [iterator, next, into_iter] == [None; 3] {
        return Ok([None; 2]);
    }
    if iterator.is_none()
        || next.is_none()
        || into_iter.is_none()
        || required_method_owner(rows, headers, next, work)? != iterator
        || LangItem::FOR_OPTION_ROLES
            .iter()
            .any(|role| roles[role.index()].is_none())
    {
        return Err("source iteration requires its actual Iterator, IntoIterator and Option");
    }
    let into = required_method_owner(rows, headers, into_iter, work)?;
    if iterator == into {
        return Err("source iteration requires distinct actual trait owners");
    }
    Ok([iterator, into])
}

fn protocol_role_header(
    rows: &SourceRows,
    role: LangItem,
    header: Option<super::super::HeaderRef>,
) -> Result<(), &'static str> {
    use super::super::HeaderRef;
    match (role, header) {
        (LangItem::TryTrait | LangItem::Iterator, Some(HeaderRef::Trait(_)))
        | (
            LangItem::ControlFlowContinue | LangItem::ControlFlowBreak,
            Some(HeaderRef::Variant(_)),
        ) => Ok(()),
        (
            LangItem::TryFromOutput
            | LangItem::TryBranch
            | LangItem::TryFromResidual
            | LangItem::IteratorNext
            | LangItem::IntoIter,
            Some(HeaderRef::Function(index)),
        ) if rows.functions.get(index as usize).is_some_and(|function| {
            function.parent.is_some() && matches!(function.body, FunctionBody::Absent)
        }) =>
        {
            Ok(())
        }
        (LangItem::IntoTryType, Some(HeaderRef::Function(index)))
            if rows.functions.get(index as usize).is_some_and(|function| {
                function.parent.is_none() && matches!(function.body, FunctionBody::Original(_))
            }) =>
        {
            Ok(())
        }
        _ if LangItem::FOR_PROTOCOL.contains(&role) => {
            Err("source iteration role has a different actual declaration header")
        }
        _ => Err("source Try role has a different actual declaration header"),
    }
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
    logical_discriminant::owned_members(
        rows,
        headers,
        [
            declarations[LangItem::DiscriminantKind.index()],
            declarations[LangItem::DiscriminantType.index()],
        ],
        work,
    )?;
    layout_members(
        rows,
        headers,
        work,
        [
            declarations[LangItem::MemSizeConst.index()],
            declarations[LangItem::MemAlignConst.index()],
        ],
    )?;
    layout_members(
        rows,
        headers,
        work,
        [declarations[LangItem::FieldOffset.index()], None],
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
    bootstrap: bool,
) -> Result<(), &'static str> {
    if profile == BindingProfile::BodyOperations {
        super::super::const_defaults::validate(headers, work)?;
    }
    for slot in &rows.slots {
        work.charge(1)?;
        if let super::super::ParameterKind::Const { ty } = slot.kind {
            use super::super::ConstFormalType;
            if slot.default.is_some() {
                return Err("source const parameter slot has a default");
            }
            if let ConstFormalType::Nominal(owner) =
                super::super::const_formal_type(&rows.types, ty)?
            {
                if profile != BindingProfile::BodyOperations {
                    return Err("source nominal const formal requires body operations");
                }
                if !matches!(
                    headers.header(owner, work)?,
                    Some(super::super::HeaderRef::Nominal(_))
                ) || headers.scopes().complete_slots(owner, work)? != 0
                {
                    return Err("source nominal const formal requires its closed nominal header");
                }
            }
        }
        if let Some(default) = slot.default {
            use super::super::{HeaderRef, ParameterKind};
            match headers.header(slot.owner, work)? {
                Some(HeaderRef::Trait(index)) => {
                    let wide = profile == BindingProfile::BodyOperations
                        && matches!(slot.kind, ParameterKind::Type { .. });
                    if !profile.languages()
                        || (!wide && rows.traits[index as usize].self_type != default)
                    {
                        return Err(
                            "source parameter default other than its trait's Self requires a wider consumer",
                        );
                    }
                }
                Some(HeaderRef::Nominal(_))
                    if profile == BindingProfile::BodyOperations
                        && matches!(slot.kind, ParameterKind::Type { .. }) => {}
                _ => return Err("source parameter default belongs to a non-trait owner"),
            }
        }
        if let super::super::ParameterKind::Type { sized } = slot.kind {
            use super::super::TypeSizing;
            match sized {
                TypeSizing::Bootstrap if !bootstrap => {
                    return Err("bootstrap sizing requires its crate body profile");
                }
                TypeSizing::Source(address)
                    if address.artifact != super::super::ArtifactRef(0)
                        || Some(address.declaration) != sized_role =>
                {
                    return Err(
                        "source sized formal names a foreign or different actual Sized role",
                    );
                }
                _ => {}
            }
        }
    }
    Ok(())
}

fn require_simple_formals(
    rows: &SourceRows,
    headers: &HeaderIndex<'_>,
    work: &mut Work,
) -> Result<(), &'static str> {
    work.charge(rows.declarations.len())?;
    for (index, row) in rows.declarations.iter().enumerate() {
        if row.definition.kind != super::super::DefinitionKind::ValueParameter {
            continue;
        }
        let declaration = crate::support::crates::owner_catalog::DeclarationRef(
            u32::try_from(index).map_err(|_| "source formal declaration identity overflow")?,
        );
        let Some(parent) = row.definition.parent else {
            return Err("source destructured parameters require the declaration policy profile");
        };
        let Some(super::super::HeaderRef::Function(fn_index)) = headers.header(parent, work)?
        else {
            return Err("source destructured parameters require the declaration policy profile");
        };
        let formals = rows.functions[fn_index as usize]
            .formals
            .slice(&rows.formals)?;
        work.charge(formals.len())?;
        if !formals
            .iter()
            .any(|formal| formal.declaration == Some(declaration))
        {
            return Err("source destructured parameters require the declaration policy profile");
        }
    }
    Ok(())
}

fn admitted_convention(
    rows: &SourceRows,
    declaration: crate::support::crates::owner_catalog::DeclarationRef,
    convention: Convention,
    profile: NamespaceProfile,
    declaration_policies: bool,
) -> bool {
    convention == Convention::Rust
        || (matches!(
            profile,
            NamespaceProfile::Definitions
                | NamespaceProfile::Callables
                | NamespaceProfile::Safety
                | NamespaceProfile::Conversions
                | NamespaceProfile::CompilerOperations
        ) && (convention == Convention::C
            || declaration_policies && convention == Convention::Unadjusted)
            && rows
                .exact_callables
                .binary_search_by_key(&declaration.0, |row| row.declaration.0)
                .is_ok())
}

#[cfg(test)]
#[path = "../../../../tests/invariants/valued_slots.rs"]
mod const_tests;

/// Whether this profile supplies the complete vocabulary for an original value.
fn original_value(
    kind: ValueKind,
    features: bool,
    regional: bool,
    constant_initializers: bool,
) -> bool {
    matches!(
        kind,
        ValueKind::Unit | ValueKind::Read(Place::Local(_)) | ValueKind::Constrain(_)
    ) || features
        && matches!(
            kind,
            ValueKind::Function(_)
                | ValueKind::Call { .. }
                | ValueKind::Scalar(_)
                | ValueKind::Aggregate { .. }
        )
        || regional
            && matches!(
                kind,
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
        || constant_initializers
            && matches!(
                kind,
                ValueKind::Read(Place::Static(_))
                    | ValueKind::AssemblyOutput { .. }
                    | ValueKind::ConstArgument(_)
                    | ValueKind::PointerCast(_)
                    | ValueKind::DirectEnumCast(_)
                    | ValueKind::AddressCast { .. }
                    | ValueKind::Unary { .. }
                    | ValueKind::Address(_)
                    | ValueKind::Length(_)
                    | ValueKind::Validate(_)
                    | ValueKind::TextEquals { .. }
                    | ValueKind::Union { .. }
                    | ValueKind::Repeat { .. }
                    | ValueKind::Unsize { .. }
                    | ValueKind::InlineConstant(_)
            )
}
