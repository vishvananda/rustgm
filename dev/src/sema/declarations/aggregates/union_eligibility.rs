//! Source union fields use the pinned trivial-copy shortcut, then an actual marker goal.
use super::{Checker, Error, ItemId, NodeKey};
use crate::resolve::{body_origin::DeclarationOrigin, model::DefId};
use crate::{
    sema::{model::Declarations, nominals::Proof},
    support::{bindings::LangItem, hash::HashSet, retained::Growth},
    types::{
        GenericArgs, TypeId,
        arguments::Argument,
        outlives::Queries,
        parameters::predicates::{Bound, Predicate},
        traits::Application,
    },
};
impl Checker<'_> {
    pub(super) fn validate_source_union_field(
        &mut self,
        ty: TypeId,
        source: NodeKey,
        proof: &mut Proof,
        proved: &mut HashSet<TypeId>,
    ) -> Result<(), Error> {
        let ty = proof
            .normalize(&mut self.facts, ty)
            .map_err(|e| self.error(source, e))?;
        crate::sema::header_attributes::charge_source_contract_query(&self.facts, 1)?;
        self.facts.union_work.steps += 1;
        if proved.contains(&ty) {
            return Ok(());
        }
        let trivial = self
            .facts
            .types
            .trivial_copy(ty)
            .ok_or_else(|| self.error(source, "missing completed union shape fact"))?;
        self.facts.union_work.unique_types += 1;
        if !trivial
            && !prove_nontrivial_field(&mut self.facts, proof, ty)
                .map_err(|error| self.error(source, error))?
        {
            return Err(self.error(
                source,
                "source union field requires its actual NoDrop or Copy proof",
            ));
        }
        let growth = Growth::map::<TypeId, ()>(proved.len(), proved.capacity(), 1);
        crate::sema::header_attributes::charge_source_retention(
            &self.facts,
            1 + growth.work,
            growth.bytes,
        )?;
        proved
            .try_reserve(1)
            .map_err(|_| self.error(source, "union field proof allocation failed"))?;
        proved.insert(ty);
        Ok(())
    }
}

/// Reuse the actual `NoDrop` marker when the provider has one, or its `Copy`
/// declaration on the smaller source profiles. A role supplies no proof.
fn prove_nontrivial_field(
    facts: &mut Declarations,
    proof: &mut Proof,
    ty: TypeId,
) -> Result<bool, &'static str> {
    let role = facts
        .source_bindings
        .as_deref()
        .ok_or("missing source union catalog")?
        .validated_role(LangItem::BikeshedGuaranteedNoDrop)?;
    let definition = if let Some(role) = role {
        if matches!(role.origin, DeclarationOrigin::Imported(_)) {
            facts.associated.require_imported_role(role)?;
        }
        role.definition
    } else {
        crate::sema::compiler_items::copy_declaration(facts)?
            .ok_or("source union field requires actual Copy or NoDrop")?
    };
    let application = facts.types.intern_trait_application(Application {
        item: ItemId(definition.0),
        arguments: GenericArgs::EMPTY,
    })?;
    proof.prove(
        facts,
        Predicate {
            ty,
            bound: Bound::Trait(application),
        },
    )
}

impl Checker<'_> {
    /// Imported overlapping storage is usable only after rechecking all its
    /// field templates with the same eligibility rule under validated impls.
    pub(in crate::sema::declarations) fn validate_imported_union_fields(
        &mut self,
    ) -> Result<(), Error> {
        let facts = &mut self.facts;
        if !facts.aggregates.has_unions() || self.report.package.source_client().is_none() {
            return Ok(());
        }
        let count = facts.aggregates.iter().len();
        crate::sema::crates::charge_source_headers(
            facts,
            count,
            count.saturating_mul(size_of::<DefId>()),
        )?;
        let linkage = facts
            .crates
            .as_deref()
            .ok_or_else(|| Error::unlocated("imported union fields lack linkage"))?;
        let mut rows = Vec::with_capacity(count);
        rows.extend(facts.aggregates.iter().filter_map(|shape| {
            let owner = DefId(shape.item.0);
            (shape.kind == crate::types::aggregate::Kind::Union && linkage.imported_record(owner))
                .then_some(owner)
        }));
        for owner in rows {
            self.validate_imported_union(owner)?;
        }
        Ok(())
    }
}

impl Checker<'_> {
    fn validate_imported_union(&mut self, owner: DefId) -> Result<(), Error> {
        let facts = &mut self.facts;
        let environment = facts
            .generics
            .environments
            .get(&owner)
            .cloned()
            .ok_or_else(|| Error::unlocated("imported union lost its declaration environment"))?;
        let universals = u32::try_from(
            facts
                .generics
                .parameters_in_scope(owner)
                .filter(|parameter| matches!(parameter.argument, Argument::Lifetime(_)))
                .count(),
        )
        .map_err(|_| Error::unlocated("imported union has too many lifetimes"))?;
        let mut proof = Proof::for_imported_declaration(environment, universals);
        imported_fields(facts, owner, &mut proof).map_err(Error::unlocated)?;
        if let Some(proof) = proof.take_trait_proof() {
            let work = {
                let mut queries = Queries::new(
                    &self.facts.types,
                    &self.facts.lifetimes.nominal_requirements,
                    &self.facts.lifetimes.callables,
                );
                let mut constraints = self
                    .trait_region_environment_count(owner, &mut queries, universals)
                    .map_err(Error::unlocated)?;
                proof
                    .require(&mut constraints, &mut queries)
                    .map_err(Error::unlocated)?;
                constraints.solve().map_err(|e| {
                    Error::unlocated(format!("imported union field lifetime requirement: {e}"))
                })?;
                queries.steps()
            };
            crate::sema::crates::pay_source_headers(&mut self.facts, work, 0)
                .map_err(Error::unlocated)?;
        }
        Ok(())
    }
}

fn imported_fields(
    facts: &mut Declarations,
    owner: DefId,
    proof: &mut Proof,
) -> Result<(), &'static str> {
    let &[variant] = facts.members.variants(ItemId(owner.0)) else {
        return Err("imported union lost its actual field group");
    };
    let variant = facts.members.variants[variant.0 as usize];
    for index in variant.start..variant.end {
        let ty = facts.members.fields[index as usize].ty;
        let properties = proof.check(facts, ty)?;
        if !properties.well_formed || !properties.sized {
            return Err("imported union requires sized well-formed fields");
        }
        let ty = proof.normalize(facts, ty)?;
        let trivial = facts
            .types
            .trivial_copy(ty)
            .ok_or("imported union field lost its checked type")?;
        if !trivial && !prove_nontrivial_field(facts, proof, ty)? {
            return Err("imported union field requires its actual NoDrop or Copy proof");
        }
    }
    crate::sema::crates::pay_source_headers(
        facts,
        proof
            .own_work()
            .saturating_add((variant.end - variant.start) as usize),
        0,
    )
}
