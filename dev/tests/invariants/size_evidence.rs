//! Hidden storage conditions retain selection lifetime debts in their parent proof.
use super::{Decision, Frame, Predicate, Search, tests};
use crate::{
    sema::{
        model::Declarations,
        traits::{assumptions::Assumptions, implementations::Availability, selection},
    },
    types::{
        GenericArgs, ItemId, Kind, Primitive, RegionId, TypeId, arguments::Argument,
        lifetime_proof::Obligations, outlives::Queries, parameters::predicates::Bound,
        region_graph::Error, relations::Constraints,
    },
};

fn valid(facts: &Declarations, proof: &Obligations) -> Result<bool, Box<dyn std::error::Error>> {
    let mut constraints = Constraints::new(&facts.types, &facts.lifetimes.variances, 1, 100_000)?;
    let input = RegionId(10);
    let universal = RegionId::bound(0, 0)?;
    constraints.variable(input)?;
    constraints.outlives(input, universal, 0)?;
    constraints.outlives(universal, input, 0)?;
    let mut queries = Queries::new(
        &facts.types,
        &facts.lifetimes.nominal_requirements,
        &facts.lifetimes.callables,
    );
    proof.require(&mut constraints, &mut queries)?;
    match constraints.solve() {
        Ok(_) => Ok(true),
        Err(Error::Unsatisfied { .. } | Error::Verification { .. }) => Ok(false),
        Err(error) => Err(error.into()),
    }
}

#[test]
fn hidden_sized_projection_preserves_each_callers_lifetime_debt()
-> Result<(), Box<dyn std::error::Error>> {
    for (bound, expected) in [("", true), (":'static", false)] {
        let source = format!(
            "trait A{{type Out:?Sized;}}impl<'a{bound}>A for &'a u8{{type Out=u8;}}
             struct S<T:A>{{tail:T::Out}}trait B{{}}impl<T>B for T{{}}"
        );
        let (report, mut facts) = tests::declarations(&source)?;
        let reference = facts.types.intern(Kind::Reference {
            mutable: false,
            region: RegionId(10),
            referent: TypeId::primitive(Primitive::U8),
        })?;
        let arguments = facts.types.intern_arguments(&[Argument::Type(reference)])?;
        let wrapper = facts.types.intern(Kind::Adt(
            ItemId(tests::definition(&report, "S")?.0),
            arguments,
        ))?;
        let blanket = tests::predicate(
            &mut facts,
            tests::definition(&report, "B")?,
            wrapper,
            GenericArgs::EMPTY,
        )?;
        for predicate in [
            Predicate {
                ty: wrapper,
                bound: Bound::Sized,
            },
            blanket,
        ] {
            let mut reuse = selection::Reuse::default();
            for first in [100, 200] {
                let (mut context, goal) = tests::instantiate(&mut facts, predicate)?;
                let assumptions = Assumptions::default();
                let mut caller = tests::caller(&assumptions, None);
                caller.first_region = first;
                let mut search = Search::new(caller, Availability::Validated);
                let frame = Frame::new(&facts, &mut context, &goal, first)?;
                let Decision::Proven(root) = search.run(&mut facts, frame)? else {
                    return Err("missing conditional sizedness proof".into());
                };
                let evidence = search.evidence(root)?;
                assert_eq!(evidence.proof.regions.start, first);
                assert!(!evidence.proof.is_empty());
                assert_eq!(valid(&facts, &evidence.proof)?, expected);

                let (mut context, goal) = tests::instantiate(&mut facts, predicate)?;
                let mut caller = tests::caller(&assumptions, None);
                caller.first_region = first;
                let selection::Selection::Proven(evidence) = selection::select_reusing(
                    &mut facts,
                    &mut context,
                    &goal,
                    caller,
                    (Availability::Validated, false),
                    &mut reuse,
                )?
                else {
                    return Err("missing conditional reused proof".into());
                };
                assert_eq!(evidence.proof.regions.start, first);
                assert!(!evidence.proof.is_empty());
                assert_eq!(valid(&facts, &evidence.proof)?, expected);
            }
        }
    }
    Ok(())
}
