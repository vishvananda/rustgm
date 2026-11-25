//! Canonical conformance keeps actual trait/member links and typed impl Self.
use super::{Declarations, DefId, OwnerKind, Report, Stage, g, s};
use crate::{
    support::constness::Constness,
    types::{Kind, TypeId},
};

fn core_builtin_inherent(report: &Report, facts: &Declarations, self_type: TypeId) -> bool {
    report.package.coherence_is_core()
        && matches!(
            facts.types.kind(self_type),
            Some(
                Kind::Primitive(_)
                    | Kind::Never
                    | Kind::Slice(_)
                    | Kind::Array { .. }
                    | Kind::ArrayTerm { .. }
                    | Kind::Pointer { .. }
            )
        )
}

impl Stage {
    pub(super) fn implementations(
        &mut self,
        report: &Report,
        facts: &Declarations,
    ) -> Result<(), &'static str> {
        let mut count = 0usize;
        let declarations = facts
            .generics
            .implementations
            .iter()
            .flat_map(|registry| registry.checked_declarations());
        for actual in declarations {
            self.charge(1)?;
            let owner = facts
                .associated
                .owner(actual.owner)
                .and_then(|owner| facts.associated.get(owner))
                .ok_or("source implementation has no actual associated owner")?;
            let owner = owner.source_owner()?;
            if owner.self_definition != actual.owner || Some(owner.source) != actual.source {
                return Err("source implementation changed its defining Self or source owner");
            }
            if !actual.types.is_empty() && !self.body_nodes()
                || !actual.constants.is_empty() && !self.declaration_policies()
            {
                return Err(
                    "source associated type/value implementation transport is not implemented",
                );
            }
            self.implementation(facts, &owner, actual)?;
            count = count
                .checked_add(1)
                .ok_or("source implementation count overflow")?;
        }
        if self.body_nodes() {
            count = count
                .checked_add(self.inherent_implementations(report, facts)?)
                .ok_or("source implementation count overflow")?;
        }
        self.charge(facts.associated.owners().len())?;
        if facts
            .associated
            .owners()
            .iter()
            .filter(|owner| matches!(owner.kind, OwnerKind::Impl { .. }))
            .count()
            != count
        {
            return Err("source impl owner has no completed canonical conformance record");
        }
        Ok(())
    }

    /// Version thirteen: an inherent impl on a provider nominal is a header
    /// without a trait application whose links name only its own methods.
    /// It follows every trait impl, so declared results keep their order.
    fn inherent_implementations(
        &mut self,
        report: &Report,
        facts: &Declarations,
    ) -> Result<usize, &'static str> {
        let mut count = 0usize;
        for (index, owner) in facts.associated.owners().iter().enumerate() {
            self.charge(1)?;
            let OwnerKind::Impl {
                trait_type: None,
                unsafe_,
                constant,
                polarity,
                ..
            } = owner.kind
            else {
                continue;
            };
            let owner = owner.source_owner()?;
            if unsafe_
                || (constant && !self.declaration_policies())
                || polarity != crate::sema::associated::ImplPolarity::Positive
            {
                return Err("source inherent impl modifiers require a wider profile");
            }
            let id = crate::sema::associated::OwnerId(
                u32::try_from(index).map_err(|_| "source owner identity overflow")?,
            );
            let methods = facts
                .associated
                .owner_members(id)
                .ok_or("source inherent impl has no actual member inventory")?;
            self.inherent_implementation(report, facts, &owner, constant, methods)?;
            count = count
                .checked_add(1)
                .ok_or("source implementation count overflow")?;
        }
        Ok(count)
    }

    fn inherent_implementation(
        &mut self,
        report: &Report,
        facts: &Declarations,
        owner: &crate::sema::associated::SourceOwner,
        constant: bool,
        methods: &[DefId],
    ) -> Result<(), &'static str> {
        let declaration = self.declaration(owner.self_definition)?;
        let schema = facts
            .associated
            .schema(owner.self_definition)
            .ok_or("source implementation has no completed typed Self schema")?;
        let core_primitive_inherent =
            self.declaration_policies() && core_builtin_inherent(report, facts, schema.self_type);
        if !core_primitive_inherent {
            let target = match facts.types.kind(schema.self_type) {
                Some(Kind::Adt(nominal, _)) => Some(DefId(nominal.0)),
                Some(Kind::Dynamic { shape, .. }) if self.declaration_policies() => facts
                    .types
                    .dynamic_shape(shape)
                    .and_then(|actual| actual.principal)
                    .map(|item| DefId(item.0)),
                _ => None,
            };
            if target.is_none_or(|head| self.declaration(head).is_err()) {
                return Err("source inherent impl requires a provider nominal Self");
            }
        }
        let context = self.frame(
            declaration,
            None,
            s::BinderKind::DeclarationHeader,
            schema.captures,
        )?;
        let self_type = self.ty(facts, schema.self_type, context)?;
        let start = u32::try_from(self.rows.clauses.len())
            .map_err(|_| "source impl input table overflow")?;
        for input in crate::sema::declaration_inputs::own(
            &facts.generics,
            facts.crates.as_deref(),
            owner.self_definition,
        )? {
            self.clause(facts, input, context, None)?;
        }
        let inputs = g::Range {
            start,
            end: u32::try_from(self.rows.clauses.len())
                .map_err(|_| "source impl input table overflow")?,
        };
        let region_bounds = self.region_bounds(&schema.bounds, context)?;
        let start = u32::try_from(self.rows.member_links.len())
            .map_err(|_| "source impl member table overflow")?;
        self.reserve(methods.len().saturating_mul(4 * size_of::<s::MemberLink>()))?;
        for &method in methods {
            self.charge(1)?;
            let admitted = self
                .functions
                .headers
                .contains_key(&crate::types::ItemId(method.0))
                || self.inherent_constant(report, facts, method).is_some()
                || self.has_constant_original(method)?;
            if !admitted {
                return Err(
                    "source inherent associated constants and types require a wider profile",
                );
            }
            let implementation = Some(self.declaration(method)?);
            self.rows.member_links.push(s::MemberLink {
                trait_member: None,
                implementation,
            });
        }
        let members = g::Range {
            start,
            end: u32::try_from(self.rows.member_links.len())
                .map_err(|_| "source impl member table overflow")?,
        };
        self.reserve(4 * size_of::<s::ImplHeader>())?;
        self.rows.implementations.push(s::ImplHeader {
            self_definition: declaration,
            scope: declaration,
            self_type,
            trait_application: None,
            unsafe_: false,
            core_primitive_inherent,
            constant,
            polarity: crate::sema::associated::ImplPolarity::Positive,
            region_captures: schema.captures,
            region_bounds,
            inputs,
            declared_inputs: u32::try_from(
                facts
                    .generics
                    .declared_predicates(owner.self_definition)
                    .len(),
            )
            .map_err(|_| "source impl declared input count overflow")?,
            members,
        });
        Ok(())
    }

    fn implementation(
        &mut self,
        facts: &Declarations,
        owner: &crate::sema::associated::SourceOwner,
        actual: &crate::sema::traits::implementations::Implementation,
    ) -> Result<(), &'static str> {
        let predicate = actual.predicate;
        let OwnerKind::Impl {
            unsafe_,
            constant,
            polarity,
            ..
        } = owner.kind
        else {
            return Err("source conformance record does not belong to an implementation");
        };
        if polarity == crate::sema::associated::ImplPolarity::Reservation
            && !self.declaration_policies()
        {
            return Err("reservation implementation transport requires a wider metadata profile");
        }
        let declaration = self.declaration(owner.self_definition)?;
        let schema = facts
            .associated
            .schema(owner.self_definition)
            .ok_or("source implementation has no completed typed Self schema")?;
        if schema.self_type != predicate.ty {
            return Err("source implementation predicate has a different actual Self type");
        }
        let context = self.frame(
            declaration,
            None,
            s::BinderKind::DeclarationHeader,
            schema.captures,
        )?;
        let self_type = self.ty(facts, schema.self_type, context)?;
        let (application, effect) = predicate
            .bound
            .trait_parts()
            .ok_or("source implementation has no actual trait application")?;
        if effect
            != if constant {
                Constness::Const
            } else {
                Constness::Ordinary
            }
        {
            return Err("source implementation effect differs from its actual const declaration");
        }
        let trait_application =
            Some(self.trait_application(facts, predicate.ty, application, context)?);
        let start = u32::try_from(self.rows.clauses.len())
            .map_err(|_| "source impl input table overflow")?;
        for input in crate::sema::declaration_inputs::own(
            &facts.generics,
            facts.crates.as_deref(),
            owner.self_definition,
        )? {
            self.clause(facts, input, context, None)?;
        }
        let inputs = g::Range {
            start,
            end: u32::try_from(self.rows.clauses.len())
                .map_err(|_| "source impl input table overflow")?,
        };
        let region_bounds = self.region_bounds(&schema.bounds, context)?;
        let members = self.implementation_members(&actual.methods, trait_application)?;
        self.implementation_results(facts, &actual.types, context, members)?;
        self.implementation_constants(&actual.constants)?;
        let members = g::Range {
            start: members.start,
            end: u32::try_from(self.rows.member_links.len())
                .map_err(|_| "source impl member table overflow")?,
        };
        self.reserve(4 * size_of::<s::ImplHeader>())?;
        self.rows.implementations.push(s::ImplHeader {
            self_definition: declaration,
            scope: declaration,
            self_type,
            trait_application,
            unsafe_,
            core_primitive_inherent: false,
            constant,
            polarity,
            region_captures: schema.captures,
            region_bounds,
            inputs,
            declared_inputs: u32::try_from(
                facts
                    .generics
                    .declared_predicates(owner.self_definition)
                    .len(),
            )
            .map_err(|_| "source impl declared input count overflow")?,
            members,
        });
        Ok(())
    }
    /// Link each checked type definition to its declared result. Version
    /// fourteen also retains the member's own lifetime frame when present.
    fn implementation_results(
        &mut self,
        facts: &Declarations,
        types: &[(DefId, DefId)],
        context: super::canonical_nodes::Context,
        members: g::Range,
    ) -> Result<(), &'static str> {
        if members.end as usize != self.rows.member_links.len() {
            return Err("source impl result links must follow their method links");
        }
        self.reserve(types.len().saturating_mul(
            4 * (size_of::<s::MemberLink>() + size_of::<s::AssociatedDefinition>()),
        ))?;
        for &(member, definition) in types {
            self.charge(1)?;
            let value = facts
                .definitions
                .get(definition.0 as usize)
                .copied()
                .flatten()
                .ok_or("source impl result has no completed definition type")?;
            let trait_member = Some(self.address(member)?);
            let declaration = self.declaration(definition)?;
            let family = facts.associated.families.get(&definition);
            let context =
                self.result_definition_context(facts, definition, declaration, context, family)?;
            if let Some(family) = family {
                let inputs =
                    self.associated_family_inputs(facts, definition, Some(family), context)?;
                if inputs.start != inputs.end {
                    self.reserve(4 * size_of::<s::AssociatedDefinitionInputs>())?;
                    let cold = self.cold()?;
                    let index = u32::try_from(cold.associated_definitions.len())
                        .map_err(|_| "source associated definition index overflow")?;
                    cold.associated_definition_inputs
                        .push(s::AssociatedDefinitionInputs { index, inputs });
                }
            }
            let value = self.ty(facts, value, context)?;
            self.rows.member_links.push(s::MemberLink {
                trait_member,
                implementation: Some(declaration),
            });
            self.cold()?
                .associated_definitions
                .push(s::AssociatedDefinition { declaration, value });
        }
        Ok(())
    }
    fn result_definition_context(
        &mut self,
        facts: &Declarations,
        definition: DefId,
        declaration: super::DeclarationRef,
        parent: super::canonical_nodes::Context,
        family: Option<&crate::sema::associated::Family>,
    ) -> Result<super::canonical_nodes::Context, &'static str> {
        let Some(family) = family else {
            return Ok(parent);
        };
        let own = facts
            .generics
            .parameters
            .get(&definition)
            .ok_or("source family definition lost its actual formals")?;
        let own_regions = own
            .iter()
            .filter(|parameter| {
                matches!(
                    parameter.argument,
                    crate::types::arguments::Argument::Lifetime(_)
                )
            })
            .count();
        let binder = self.binder(parent)?;
        if !self.declaration_policies()
            || own.is_empty()
            || !family.regions.is_empty()
            || family.requirements.is_none()
            || family.captures != self.rows.binders[binder.0 as usize].count
            || family.captures as usize + own_regions != family.total as usize
        {
            return Err("source family definition requires its wider parameter or premise profile");
        }
        self.frame(
            declaration,
            None,
            s::BinderKind::DeclarationHeader,
            family.total,
        )
    }
    fn implementation_members(
        &mut self,
        methods: &[(DefId, Option<DefId>)],
        trait_application: Option<s::TraitApplicationRef>,
    ) -> Result<g::Range, &'static str> {
        let start = u32::try_from(self.rows.member_links.len())
            .map_err(|_| "source impl member table overflow")?;
        self.reserve(methods.len().saturating_mul(4 * size_of::<s::MemberLink>()))?;
        for &(member, implementation) in methods {
            self.charge(1)?;
            if let Some(actual) = implementation
                && self.inputs.selected_methods.is_some()
            {
                self.reserve(crate::support::retained::new_map::<
                    DefId,
                    s::TraitApplicationRef,
                >(1))?;
                let map = self
                    .inputs
                    .selected_methods
                    .as_mut()
                    .ok_or("selected source map disappeared")?;
                if map
                    .insert(
                        actual,
                        trait_application.ok_or("selected method lost its trait application")?,
                    )
                    .is_some()
                {
                    return Err("selected source method belongs to multiple actual impls");
                }
            }
            self.implementation_member_link(member, implementation)?;
        }
        Ok(g::Range {
            start,
            end: u32::try_from(self.rows.member_links.len())
                .map_err(|_| "source impl member table overflow")?,
        })
    }
    fn implementation_constants(
        &mut self,
        constants: &[(DefId, Option<DefId>)],
    ) -> Result<(), &'static str> {
        if constants.is_empty() {
            return Ok(());
        }
        self.reserve(
            constants
                .len()
                .saturating_mul(4 * size_of::<s::MemberLink>()),
        )?;
        for &(member, implementation) in constants {
            self.charge(1)?;
            self.implementation_member_link(member, implementation)?;
        }
        Ok(())
    }
    fn implementation_member_link(
        &mut self,
        member: DefId,
        implementation: Option<DefId>,
    ) -> Result<(), &'static str> {
        let trait_member = Some(self.address(member)?);
        let implementation = implementation
            .map(|definition| self.declaration(definition))
            .transpose()?;
        self.rows.member_links.push(s::MemberLink {
            trait_member,
            implementation,
        });
        Ok(())
    }
}
