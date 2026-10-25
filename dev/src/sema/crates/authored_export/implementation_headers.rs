//! Canonical conformance keeps actual trait/member links and typed impl Self.
use super::{Declarations, DefId, OwnerKind, Stage, g, s};
use crate::{support::constness::Constness, types::parameters::predicates::Predicate};

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
            self.implementation(facts, &owner, actual.predicate, &actual.methods)?;
            if !actual.types.is_empty() || !actual.constants.is_empty() {
                return Err(
                    "source associated type/value implementation transport is not implemented",
                );
            }
            count = count
                .checked_add(1)
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

    fn implementation(
        &mut self,
        facts: &Declarations,
        owner: &crate::sema::associated::SourceOwner,
        predicate: Predicate,
        methods: &[(DefId, Option<DefId>)],
    ) -> Result<(), &'static str> {
        let OwnerKind::Impl {
            unsafe_,
            constant,
            negative,
            ..
        } = owner.kind
        else {
            return Err("source conformance record does not belong to an implementation");
        };
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
        let start = u32::try_from(self.rows.member_links.len())
            .map_err(|_| "source impl member table overflow")?;
        self.reserve(methods.len().saturating_mul(4 * size_of::<s::MemberLink>()))?;
        for &(member, implementation) in methods {
            self.charge(1)?;
            let trait_member = Some(self.address(member)?);
            let implementation = implementation
                .map(|definition| self.declaration(definition))
                .transpose()?;
            self.rows.member_links.push(s::MemberLink {
                trait_member,
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
            trait_application,
            unsafe_,
            constant,
            negative,
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
}
