//! Caller quantifiers preserve input correlations without introducing new premises.
#[path = "caller_outlives/correlations.rs"]
mod correlations;
#[path = "caller_outlives/contract_formation.rs"]
mod formation;
#[path = "caller_outlives/universes.rs"]
mod universes;
use rustgm::types::{
    GenericArgs, ItemId, Kind, RegionId, Store, TypeId,
    arguments::Argument,
    callable::Templates,
    copy::Properties,
    outlives::{Bounds, Queries},
    parameters::{
        Parameter, ParameterId, ParameterKind,
        clauses::{Clause, Quantified},
        predicates::{Bound, Predicate},
    },
    projection::{Application, Schema},
    relations::{Constraints, ProofScope},
    traits,
    variance::Variances,
};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Default)]
struct Fixture {
    types: Store,
    bounds: Bounds,
    templates: Templates,
    variances: Variances,
    parameters: Vec<ParameterId>,
}
impl Fixture {
    fn parameter(&mut self) -> Result<TypeId> {
        let id = self.types.intern_parameter(Parameter {
            owner: ItemId(100),
            index: u32::try_from(self.parameters.len())?,
            kind: ParameterKind::Type { sized: true },
        })?;
        self.parameters.push(id);
        Ok(self.types.intern(Kind::Parameter(id))?)
    }
    fn reference(&mut self, region: RegionId, referent: TypeId) -> Result<TypeId> {
        Ok(self.types.intern(Kind::Reference {
            region,
            referent,
            mutable: false,
        })?)
    }
    fn tuple(&mut self, elements: &[TypeId]) -> Result<TypeId> {
        let elements = self.types.intern_list(elements)?;
        Ok(self.types.intern(Kind::Tuple(elements))?)
    }
    fn projection(&mut self, member: u32, subject: TypeId, inputs: &[Argument]) -> Result<TypeId> {
        self.types.declare_projection(
            ItemId(member),
            Schema {
                trait_item: ItemId(10),
                sized: true,
                copies: Properties::default(),
            },
        )?;
        let arguments = self.types.intern_arguments(inputs)?;
        let trait_application = self.types.intern_trait_application(traits::Application {
            item: ItemId(10),
            arguments,
        })?;
        Ok(self.types.intern(Kind::Projection(Application {
            item: ItemId(member),
            subject,
            trait_application,
            arguments: GenericArgs::EMPTY,
        }))?)
    }
    fn clause(&mut self, bound_regions: u32, ty: TypeId, shorter: RegionId) -> Result<Clause> {
        Ok(Clause::ForAll(self.types.intern_quantified(
            Quantified {
                bound_regions,
                predicate: Predicate {
                    ty,
                    bound: Bound::Outlives(shorter),
                },
            },
        )?))
    }
    fn proof(&self, count: u32) -> Result<(Constraints<'_>, Queries<'_>)> {
        let mut proof = Constraints::new(&self.types, &self.variances, count, 1_000_000)?;
        for &id in &self.parameters {
            proof.parameter(id)?;
        }
        Ok((
            proof,
            Queries::new(&self.types, &self.bounds, &self.templates),
        ))
    }
}

#[test]
fn direct_parameter_all_regions_bound_does_not_distribute_over_composites() -> Result {
    let mut f = Fixture::default();
    let t = f.parameter()?;
    let local = RegionId::bound(0, 0)?;
    let captured = RegionId::bound(1, 0)?;
    let reference = f.reference(local, t)?;
    let owner_reference = f.reference(captured, t)?;
    let tuple = f.tuple(&[t])?;
    for (subject, target, expected) in [
        (t, local, true),
        (tuple, local, false),
        (reference, local, false),
        (owner_reference, local, false),
        (reference, RegionId::STATIC, true),
        (tuple, RegionId::STATIC, true),
    ] {
        let clause = f.clause(1, subject, target)?;
        let (mut proof, mut queries) = f.proof(1)?;
        proof.assume_outlives_clause(&mut queries, clause)?;
        proof.require_type(&mut queries, t, RegionId::STATIC, ProofScope::ROOT, 17)?;
        assert_eq!(proof.solve().is_ok(), expected, "{subject:?}: {target:?}");
    }
    Ok(())
}

#[test]
fn parameter_permissions_survive_specialization_without_reclassifying_the_source() -> Result {
    let mut f = Fixture::default();
    let t = f.parameter()?;
    let Kind::Parameter(id) = f.types.kind(t).ok_or("missing parameter")? else {
        return Err("invalid parameter fixture".into());
    };
    let original = f.clause(1, t, RegionId::bound(0, 0)?)?;
    let premise = original
        .caller_outlives(&f.types)?
        .ok_or("missing all-regions parameter permission")?;
    let actual = f.reference(RegionId::bound(0, 0)?, TypeId::UNIT)?;
    let mut fold =
        rustgm::types::substitute::Bound::parameters(&f.types, &[(id, Argument::Type(actual))])?;
    let premise = fold.clause(&mut f.types, premise)?;
    let (mut proof, mut queries) = f.proof(1)?;
    proof.assume_outlives_clause(&mut queries, premise)?;
    proof.require_type(&mut queries, actual, RegionId::STATIC, ProofScope::ROOT, 9)?;
    proof.solve()?;

    let reference = f.reference(RegionId::bound(1, 0)?, TypeId::UNIT)?;
    let written = f.clause(1, reference, RegionId::bound(0, 0)?)?;
    assert!(written.caller_outlives(&f.types)?.is_none());
    let (mut proof, mut queries) = f.proof(1)?;
    proof.assume_outlives_clause(&mut queries, written)?;
    proof.require_type(&mut queries, actual, RegionId::STATIC, ProofScope::ROOT, 9)?;
    assert!(proof.solve().is_err());
    Ok(())
}

#[test]
fn empty_and_unused_binders_lift_only_owner_captures() -> Result {
    let mut f = Fixture::default();
    let t = f.parameter()?;
    let subject = f.reference(RegionId::bound(1, 0)?, t)?;
    for count in [0, 1, 7] {
        let clause = f.clause(count, subject, RegionId::bound(1, 1)?)?;
        let (mut proof, mut queries) = f.proof(2)?;
        proof.assume_outlives_clause(&mut queries, clause)?;
        proof.require_type(&mut queries, t, RegionId::bound(0, 1)?, ProofScope::ROOT, 0)?;
        proof.outlives(RegionId::bound(0, 0)?, RegionId::bound(0, 1)?, 0)?;
        proof.solve()?;
        let (mut proof, mut queries) = f.proof(2)?;
        proof.assume_outlives_clause(&mut queries, clause)?;
        proof.outlives(RegionId::bound(0, 1)?, RegionId::bound(0, 0)?, 0)?;
        assert!(proof.solve().is_err());
    }
    Ok(())
}

#[test]
fn elaboration_keeps_independent_parts_without_inverting_opaque_results() -> Result {
    let mut f = Fixture::default();
    let t = f.parameter()?;
    let u = f.parameter()?;
    let local = RegionId::bound(0, 0)?;
    let projected = f.projection(11, t, &[Argument::Lifetime(local)])?;
    let subject = f.tuple(&[u, projected])?;
    let clause = f.clause(1, subject, RegionId::STATIC)?;
    for (ty, expected) in [(u, true), (t, false)] {
        let (mut proof, mut queries) = f.proof(0)?;
        proof.assume_outlives_clause(&mut queries, clause)?;
        proof.require_type(&mut queries, ty, RegionId::STATIC, ProofScope::ROOT, 0)?;
        assert_eq!(proof.solve().is_ok(), expected);
    }
    Ok(())
}

#[test]
fn batched_targets_keep_distinct_permissions_and_opaque_results() -> Result {
    let mut f = Fixture::default();
    let t = f.parameter()?;
    let u = f.parameter()?;
    let projected = f.projection(11, t, &[])?;
    let local = f.reference(RegionId::bound(0, 0)?, projected)?;
    let captured = f.reference(RegionId::bound(1, 0)?, u)?;
    let shared = f.tuple(&[local, captured])?;
    let first = f.clause(1, shared, RegionId::bound(1, 1)?)?;
    let second = f.clause(0, u, RegionId::bound(1, 2)?)?;
    for (ty, target, expected) in [
        (projected, 1, true),
        (projected, 2, false),
        (u, 1, true),
        (u, 2, true),
        (t, 1, false),
    ] {
        let (mut proof, mut queries) = f.proof(3)?;
        proof.assume_outlives_clauses(&mut queries, [first, second])?;
        proof.require_type(
            &mut queries,
            ty,
            RegionId::bound(0, target)?,
            ProofScope::ROOT,
            23,
        )?;
        assert_eq!(proof.solve().is_ok(), expected, "{ty:?}: {target}");
    }
    for (target, expected) in [(1, true), (2, false)] {
        let (mut proof, mut queries) = f.proof(3)?;
        proof.assume_outlives_clauses(&mut queries, [first, second])?;
        proof.outlives(RegionId::bound(0, 0)?, RegionId::bound(0, target)?, 23)?;
        assert_eq!(proof.solve().is_ok(), expected);
    }
    // A second consumer can share immutable type facts, never the first one's permissions.
    let (mut proof, mut queries) = f.proof(3)?;
    proof.assume_outlives_clauses(&mut queries, [second])?;
    proof.require_type(
        &mut queries,
        u,
        RegionId::bound(0, 1)?,
        ProofScope::ROOT,
        23,
    )?;
    assert!(proof.solve().is_err());
    Ok(())
}

#[test]
fn repeated_clauses_are_charged_even_when_they_add_no_components() -> Result {
    let mut f = Fixture::default();
    let clause = f.clause(1, TypeId::UNIT, RegionId::STATIC)?;
    let mut proof = Constraints::new(&f.types, &f.variances, 0, 64)?;
    let mut queries = Queries::new(&f.types, &f.bounds, &f.templates);
    assert!(
        proof
            .assume_outlives_clauses(&mut queries, std::iter::repeat_n(clause, 65))
            .is_err()
    );
    Ok(())
}
