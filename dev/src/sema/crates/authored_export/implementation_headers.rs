//! Canonical conformance keeps actual trait/member links and typed impl Self.
use super::{Declarations, DefId, OwnerKind, Stage, g, s};
use crate::support::constness::Constness;

impl Stage {
    pub(super) fn implementations(&mut self, facts: &Declarations) -> Result<(), &'static str> {
        let mut count = 0usize;
        for actual in facts.generics.implementations() {
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
            if !actual.types.is_empty() && !self.body_nodes() || !actual.constants.is_empty() {
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
                .checked_add(self.inherent_implementations(facts)?)
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
    fn inherent_implementations(&mut self, facts: &Declarations) -> Result<usize, &'static str> {
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
            if unsafe_ || constant || polarity != crate::sema::associated::ImplPolarity::Positive {
                return Err("source inherent impl modifiers require a wider profile");
            }
            let id = crate::sema::associated::OwnerId(
                u32::try_from(index).map_err(|_| "source owner identity overflow")?,
            );
            let methods = facts
                .associated
                .owner_members(id)
                .ok_or("source inherent impl has no actual member inventory")?;
            self.inherent_implementation(facts, &owner, methods)?;
            count = count
                .checked_add(1)
                .ok_or("source implementation count overflow")?;
        }
        Ok(count)
    }

    fn inherent_implementation(
        &mut self,
        facts: &Declarations,
        owner: &crate::sema::associated::SourceOwner,
        methods: &[DefId],
    ) -> Result<(), &'static str> {
        let declaration = self.declaration(owner.self_definition)?;
        let schema = facts
            .associated
            .schema(owner.self_definition)
            .ok_or("source implementation has no completed typed Self schema")?;
        let Some(crate::types::Kind::Adt(nominal, _)) = facts.types.kind(schema.self_type) else {
            return Err("source inherent impl requires a provider nominal Self");
        };
        if self.declaration(DefId(nominal.0)).is_err() {
            return Err("source inherent impl requires a provider nominal Self");
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
            if !self
                .functions
                .headers
                .contains_key(&crate::types::ItemId(method.0))
            {
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
            constant: false,
            negative: false,
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
        if polarity == crate::sema::associated::ImplPolarity::Reservation {
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
            constant,
            negative: polarity == crate::sema::associated::ImplPolarity::Negative,
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
    /// Version thirteen: each checked type definition links its trait result and
    /// retains its value recipe in the implementation's own frame.
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
            let trait_member = Some(self.address(member)?);
            let implementation = implementation
                .map(|definition| self.declaration(definition))
                .transpose()?;
            self.rows.member_links.push(s::MemberLink {
                trait_member,
                implementation,
            });
        }
        Ok(g::Range {
            start,
            end: u32::try_from(self.rows.member_links.len())
                .map_err(|_| "source impl member table overflow")?,
        })
    }
}
