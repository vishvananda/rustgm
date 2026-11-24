//! Complete validated source headers retain exact Self syntax and attribute claims.
mod supertrait_sources;
use super::{
    DeclarationRef, Declarations, DefKind, OwnerKind, Report, Stage, canonical_nodes::Context, g, s,
};
use crate::{
    resolve::bindings::{HeaderAttribute, Reservations},
    sema::compiler_items::headers::Trait,
    support::bindings::LangItem,
    types::{ItemId, Kind},
};

/// Admission precedes any declaration/constant row serialization. Version
/// thirteen admits every declaration role here; each role's header form and the
/// semantic fences below still decide whether it transports.
pub(super) fn require_role_transport(
    report: &Report,
    facts: &Declarations,
    conversions: bool,
    compiler_operations: bool,
    languages: bool,
) -> Result<(), &'static str> {
    let catalog = facts
        .source_bindings
        .as_deref()
        .ok_or("source header graph has no role catalog")?;
    crate::sema::compiler_items::require_startup_transport(facts)?;
    crate::sema::compiler_items::require_begin_panic_transport(facts)?;
    crate::sema::compiler_items::require_personality_transport(facts)?;
    if LangItem::DECLARATIONS.into_iter().any(|role| {
        let future = matches!(
            role,
            LangItem::Future
                | LangItem::FutureOutput
                | LangItem::FuturePoll
                | LangItem::AsyncDropInPlace
        );
        let admitted = if future {
            compiler_operations
        } else {
            languages
        };
        catalog.language(role).is_some() && !admitted
    }) {
        return Err("source compiler language contract transport is not implemented");
    }
    if !(compiler_operations && languages)
        && [LangItem::TypeInfo, LangItem::TypeId]
            .into_iter()
            .any(|role| catalog.language(role).is_some())
    {
        return Err("source type reflection contract transport is not implemented");
    }
    if !conversions && catalog.language(LangItem::From).is_some() {
        return Err("source From contract transport is not implemented");
    }
    if !(compiler_operations && languages) && catalog.language(LangItem::Div).is_some() {
        return Err("source Div contract transport is not implemented");
    }
    if [LangItem::RangeMin, LangItem::RangeMax, LangItem::RangeSub]
        .into_iter()
        .any(|role| catalog.language(role).is_some())
    {
        if !(compiler_operations && languages) {
            return Err("source range helper contract transport is not implemented");
        }
        crate::sema::compiler_items::require_range_transport(report, facts)?;
    }
    Ok(())
}

impl Stage {
    pub(super) fn traits(
        &mut self,
        report: &Report,
        facts: &Declarations,
        ordinary_traits: bool,
    ) -> Result<(), &'static str> {
        let catalog = facts
            .source_bindings
            .as_deref()
            .ok_or("source header graph has no role catalog")?;
        self.charge(1)?;
        // Version thirteen re-proves UnsafeUnpin as an exact empty auto role.
        if !self.body_nodes() && catalog.language(LangItem::UnsafeUnpin).is_some() {
            return Err("source UnsafeUnpin contract transport is not implemented");
        }
        if catalog.language(LangItem::ConstParamTy).is_some() {
            return Err("source ConstParamTy contract transport is not implemented");
        }
        let reserved = report
            .resolution
            .source_bindings
            .as_deref()
            .ok_or("source role reservations are absent")?;
        let start =
            u32::try_from(self.rows.roles.len()).map_err(|_| "source role table overflow")?;
        let mut emitted = if ordinary_traits {
            self.reserve(report.resolution.definitions.len())?;
            vec![false; report.resolution.definitions.len()]
        } else {
            Vec::new()
        };
        for role in LangItem::ALL {
            self.charge(1)?;
            let Some(declaration) = self.admitted_role(report, facts, role)? else {
                continue;
            };
            let address = self.address(declaration.definition)?;
            self.reserve(4 * size_of::<s::RoleBinding>())?;
            self.rows.roles.push(s::RoleBinding {
                role,
                declaration: address,
            });
            if report.resolution.definitions[declaration.definition.0 as usize].kind
                == DefKind::Trait
            {
                let (header, literal) = Trait::source_with_bounds(report, facts, declaration)?;
                self.trait_header(facts, &header, reserved, literal)?;
                if let Some(slot) = emitted.get_mut(declaration.definition.0 as usize) {
                    *slot = true;
                }
            }
        }
        self.rows.policy.roles = g::Range {
            start,
            end: u32::try_from(self.rows.roles.len()).map_err(|_| "source role table overflow")?,
        };
        if ordinary_traits {
            for owner in facts.associated.owners() {
                self.charge(1)?;
                let OwnerKind::Trait { definition, .. } = owner.kind else {
                    continue;
                };
                let slot = emitted
                    .get_mut(definition.0 as usize)
                    .ok_or("ordinary trait owner is outside its actual declaration arena")?;
                if !*slot {
                    let (header, literal) = Trait::source_owner_with_bounds(report, facts, owner)?;
                    self.trait_header(facts, &header, reserved, literal)?;
                    *slot = true;
                }
            }
        }
        let expected = facts
            .associated
            .owners()
            .iter()
            .filter(|owner| matches!(owner.kind, OwnerKind::Trait { .. }))
            .count();
        self.charge(facts.associated.owners().len())?;
        if self.rows.traits.len() != expected {
            return Err("source trait graph contains an unsupported unvalidated declaration");
        }
        Ok(())
    }

    /// Version thirteen authenticates operator roles with their own contract;
    /// every other role must already be a completed header role.
    /// A non-trait role row carries only its declaration address.
    fn declared_role_form(&self, kind: DefKind, role: LangItem) -> Result<(), &'static str> {
        match kind {
            DefKind::TypeAlias if role == LangItem::FutureOutput && self.compiler_operations => {}
            DefKind::Function if role == LangItem::AsyncDropInPlace && self.compiler_operations => {
            }
            DefKind::Function if role == LangItem::FuturePoll && self.compiler_operations => {}
            DefKind::Function if role == LangItem::CloneFn => {}
            // FnOnce's declared result travels as its kind-9 result row.
            DefKind::TypeAlias if role == LangItem::FnOnceOutput && self.body_nodes() => {}
            DefKind::TypeAlias if role == LangItem::MetadataType && self.declaration_policies() => {
            }
            // The consumer re-proves the named unit struct and interns its
            // logical argument; no variance or ownership fact travels.
            DefKind::Struct if role == LangItem::PhantomData && self.body_nodes() => {}
            DefKind::Struct if role.is_storage_wrapper() && self.declaration_policies() => {}
            DefKind::Struct
                if self.declaration_policies()
                    && matches!(
                        role,
                        LangItem::TypeInfo | LangItem::TypeId | LangItem::DynMetadata
                    ) => {}
            DefKind::Const
                if self.declaration_policies()
                    && matches!(role, LangItem::RangeMin | LangItem::RangeMax) => {}
            DefKind::Function if self.declaration_policies() && role == LangItem::RangeSub => {}
            // The consumer re-proves the named declaration's installed header.
            DefKind::Function if role == LangItem::Panic => {
                if !self.body_nodes() {
                    return Err("source panic role requires the version-thirteen role profile");
                }
            }
            _ => return Err("source role form is outside the complete first transport family"),
        }
        Ok(())
    }

    /// Version thirteen transports `NonNull`'s and `Send`'s diagnostic
    /// identities as role table rows after the language rows, in the closed
    /// table's order; older profiles reject `NonNull`'s by name here and
    /// `Send`'s in the role stage. This runs after every other export stage,
    /// so an export that fails earlier keeps its own diagnostic.
    pub(super) fn diagnostic_roles(&mut self, report: &Report) -> Result<(), &'static str> {
        self.charge(1)?;
        let Some(reservations) = report.resolution.source_bindings.as_deref() else {
            return Ok(());
        };
        for (role, declaration) in [
            (
                s::DiagnosticRole::NonNull,
                Reservations::nonnull_declaration(reservations),
            ),
            (
                s::DiagnosticRole::Send,
                reservations.language(LangItem::Send),
            ),
        ] {
            let Some(declaration) = declaration else {
                continue;
            };
            if !self.body_nodes() {
                return Err(match role {
                    s::DiagnosticRole::NonNull => {
                        "source NonNull diagnostic identity transport requires version 13"
                    }
                    s::DiagnosticRole::Send => {
                        "source Send diagnostic contract transport is not implemented"
                    }
                });
            }
            let address = self.address(declaration.definition)?;
            self.reserve(4 * size_of::<s::DiagnosticBinding>())?;
            self.cold()?.diagnostics.push(s::DiagnosticBinding {
                role,
                declaration: address,
            });
        }
        Ok(())
    }

    /// Check the role producer's admission before assigning any canonical rows.
    pub(super) fn admitted_role(
        &mut self,
        report: &Report,
        facts: &Declarations,
        role: LangItem,
    ) -> Result<Option<crate::resolve::bindings::Declaration>, &'static str> {
        let catalog = facts
            .source_bindings
            .as_deref()
            .ok_or("source header graph has no role catalog")?;
        if role == LangItem::ConstParamTy && catalog.language(role).is_some() {
            return Err("source ConstParamTy contract transport is not implemented");
        }
        let Some(declaration) = self.exported_role(report, facts, role)? else {
            return Ok(None);
        };
        if (!self.declaration_policies()
            && matches!(
                role,
                LangItem::PointeeTrait | LangItem::MetadataType | LangItem::DynMetadata
            ))
            || (role == LangItem::PhantomData && !self.body_nodes())
        {
            return Err("source Pointee contract transport is not implemented");
        }
        // Version thirteen writes Send's diagnostic identity in a separate row.
        if role == LangItem::Send {
            return if self.body_nodes() {
                Ok(None)
            } else {
                Err("source Send diagnostic contract transport is not implemented")
            };
        }
        // The historical profile treated unwind safety as ordinary auto traits.
        if matches!(role, LangItem::UnwindSafe | LangItem::RefUnwindSafe)
            && !self.declaration_policies()
        {
            return Ok(None);
        }
        let kind = report.resolution.definitions[declaration.definition.0 as usize].kind;
        if kind != DefKind::Trait {
            self.declared_role_form(kind, role)?;
        }
        Ok(Some(declaration))
    }

    fn exported_role(
        &mut self,
        report: &Report,
        facts: &Declarations,
        role: LangItem,
    ) -> Result<Option<crate::resolve::bindings::Declaration>, &'static str> {
        if self.body_nodes()
            && let Some((declaration, work)) =
                crate::sema::compiler_items::exported_operator_role(report, facts, role)?
        {
            self.charge(work)?;
            return Ok(declaration);
        }
        facts
            .source_bindings
            .as_deref()
            .ok_or("source header graph has no role catalog")?
            .header_role(role)
    }

    fn trait_header(
        &mut self,
        facts: &Declarations,
        header: &Trait<'_>,
        reserved: &Reservations,
        literal: &[crate::syntax::ast::NodeId],
    ) -> Result<(), &'static str> {
        self.charge(1)?;
        let canonical = facts
            .generics
            .traits
            .as_deref()
            .and_then(|traits| traits.header(ItemId(header.definition.0)))
            .ok_or("source trait has no complete canonical header")?;
        if !self.compiler_operations && !self.body_nodes() && !canonical.results.is_empty() {
            return Err("source associated result transport is not implemented");
        }
        let inventory =
            supertrait_sources::Inventory::new(self, facts, header, canonical, literal)?;
        let prefix = inventory.prefix;
        let declaration = self.declaration(header.definition)?;
        let self_definition = self.declaration(header.self_definition)?;
        let context = self.frame(
            declaration,
            None,
            s::BinderKind::DeclarationHeader,
            header.captures,
        )?;
        let self_type = self.ty(facts, header.self_type, context)?;
        let Kind::Parameter(parameter) = facts
            .types
            .kind(header.self_type)
            .ok_or("source trait Self type is unknown")?
        else {
            return Err("source trait lost its actual Self parameter");
        };
        let self_slot = self
            .graph
            .parameters
            .get(&parameter)
            .copied()
            .ok_or("source trait Self slot was not reserved")?;
        self.trait_defaults(facts, header.definition, header.self_type, self_type)?;
        let (inputs, written_supertraits) =
            self.trait_inputs(facts, header, context, self_slot, inventory)?;
        let region_bounds = self.region_bounds(&canonical.regions, context)?;
        let start =
            u32::try_from(self.rows.members.len()).map_err(|_| "source member table overflow")?;
        self.reserve(
            header
                .members
                .len()
                .saturating_mul(4 * size_of::<DeclarationRef>()),
        )?;
        for &member in header.members {
            let actual = self.declaration(member)?;
            self.rows.members.push(actual);
        }
        let members = g::Range {
            start,
            end: u32::try_from(self.rows.members.len())
                .map_err(|_| "source member table overflow")?,
        };
        let policy = if self.declaration_policies() {
            facts.authored_definition_policy(header.definition)?
                | facts.authored_dispatch_bits(header.definition)?
                | (u16::from(facts.authored_incoherent_target(header.definition)?)
                    * s::TraitFlags::INCOHERENT_INHERENT)
        } else {
            0
        };
        let flags = flags(header, reserved, policy)?;
        self.reserve(4 * size_of::<s::TraitHeader>())?;
        self.rows.traits.push(s::TraitHeader {
            declaration,
            scope: declaration,
            self_definition,
            self_type,
            flags,
            region_captures: header.captures,
            region_bounds,
            inputs,
            canonical_inputs: prefix.canonical,
            declared_inputs: prefix.declared,
            written_parameters: u32::try_from(header.written.parameters)
                .map_err(|_| "source written parameter count overflow")?,
            written_supertraits,
            members,
        });
        Ok(())
    }

    fn trait_inputs(
        &mut self,
        facts: &Declarations,
        header: &Trait<'_>,
        context: Context,
        self_slot: s::SlotRef,
        mut inventory: supertrait_sources::Inventory<'_>,
    ) -> Result<(g::Range, g::Range), &'static str> {
        let start =
            u32::try_from(self.rows.clauses.len()).map_err(|_| "source clause table overflow")?;
        for (ordinal, input) in header.inputs.iter().enumerate() {
            self.charge(1)?;
            let is_self = inventory.is_self(facts, ordinal)?;
            let index = self.clause(facts, input, context, is_self.then_some(self_slot))?;
            inventory.record(self, facts, ordinal, input, index, is_self)?;
        }
        let inputs = g::Range {
            start,
            end: u32::try_from(self.rows.clauses.len())
                .map_err(|_| "source clause table overflow")?,
        };
        Ok((inputs, inventory.finish(self)?))
    }

    pub(super) fn region_bounds(
        &mut self,
        bounds: &[(crate::types::RegionId, crate::types::RegionId)],
        context: Context,
    ) -> Result<g::Range, &'static str> {
        self.reserve(bounds.len().saturating_mul(4 * size_of::<s::RegionBound>()))?;
        let start = u32::try_from(self.rows.region_bounds.len())
            .map_err(|_| "source region bound table overflow")?;
        for &(longer, shorter) in bounds {
            self.charge(1)?;
            let longer = self.region(longer, context)?;
            let shorter = self.region(shorter, context)?;
            self.rows
                .region_bounds
                .push(s::RegionBound { longer, shorter });
        }
        Ok(g::Range {
            start,
            end: u32::try_from(self.rows.region_bounds.len())
                .map_err(|_| "source region bound table overflow")?,
        })
    }
}

fn flags(
    header: &Trait<'_>,
    reserved: &Reservations,
    policy: u16,
) -> Result<s::TraitFlags, &'static str> {
    if header.attributes.contains(HeaderAttribute::Marker) {
        return Err("source marker trait contract transport is not implemented");
    }
    let mut bits = (u16::from(header.flags.unsafe_) * s::TraitFlags::UNSAFE)
        | (u16::from(header.flags.constant) * s::TraitFlags::CONST)
        | (u16::from(header.flags.auto_) * s::TraitFlags::AUTO)
        | (u16::from(reserved.denies_implementation(header.definition)) * s::TraitFlags::DENY_IMPL);
    for (attribute, flag) in [
        (HeaderAttribute::Fundamental, s::TraitFlags::FUNDAMENTAL),
        (
            HeaderAttribute::Specialization,
            s::TraitFlags::SPECIALIZATION,
        ),
        (HeaderAttribute::Coinductive, s::TraitFlags::COINDUCTIVE),
        (
            HeaderAttribute::DynIncompatible,
            s::TraitFlags::DYN_INCOMPATIBLE,
        ),
    ] {
        if header.attributes.contains(attribute) {
            bits |= flag;
        }
    }
    s::TraitFlags::from_bits(bits | policy)
}
