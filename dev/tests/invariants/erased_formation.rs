//! Erasing a projected output never erases its callable-scoped formation proof.
use super::{
    Decision, Frame, Search, continuation::Progress, header_tests, quantified_tests::valid, tests,
};
use crate::{
    resolve::expansion::Report,
    sema::{
        model::Declarations,
        nominals::traits::State,
        traits::{assumptions::Assumptions, implementations::Availability},
    },
    types::{
        GenericArgs, ItemId, Kind, Primitive, RegionId, TypeId, arguments::Argument,
        parameters::predicates::Predicate, projection, traits,
    },
};

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

fn fixture(implementation: &str) -> TestResult<(Report, Declarations, Predicate)> {
    let (report, mut facts) = tests::declarations(&format!(
        "trait A<'a>{{type Out;}}{implementation}trait Accept{{}}impl Accept for for<'a>fn(&'a u8)->u8{{}}"
    ))?;
    let byte = TypeId::primitive(Primitive::U8);
    let local = RegionId::bound(0, 0)?;
    let arguments = facts.types.intern_arguments(&[Argument::Lifetime(local)])?;
    let trait_application = facts.types.intern_trait_application(traits::Application {
        item: ItemId(tests::definition(&report, "A")?.0),
        arguments,
    })?;
    let result = facts
        .types
        .intern(Kind::Projection(projection::Application {
            item: ItemId(tests::definition(&report, "Out")?.0),
            subject: byte,
            trait_application,
            arguments: GenericArgs::EMPTY,
        }))?;
    let argument = facts.types.intern(Kind::Reference {
        mutable: false,
        region: local,
        referent: byte,
    })?;
    let parameters = facts.types.intern_list(&[argument])?;
    let function = facts.types.intern(Kind::Function {
        bound_regions: 1,
        parameters,
        result,
        unsafe_: false,
        convention: crate::types::callable::Convention::RUST,
    })?;
    let goal = tests::predicate(
        &mut facts,
        tests::definition(&report, "Accept")?,
        function,
        GenericArgs::EMPTY,
    )?;
    Ok((report, facts, goal))
}

#[test]
fn erased_callable_results_retain_universal_implementation_requirements()
-> Result<(), Box<dyn std::error::Error>> {
    for (implementation, expected) in [
        ("impl<'a>A<'a>for u8{type Out=u8;}", true),
        ("impl A<'static>for u8{type Out=u8;}", false),
        ("impl<'a:'static>A<'a>for u8{type Out=u8;}", false),
    ] {
        let (_, mut facts, goal) = fixture(implementation)?;
        let mut state = State::instance(&mut facts, 0)?;
        assert!(state.prove(&mut facts, goal, true)?, "{implementation}");
        let proof = state.finish();
        assert!(proof.scope_count() > 0);
        assert_eq!(valid(&facts, &proof, 0)?, expected, "{implementation}");
    }
    Ok(())
}

#[test]
fn callable_input_header_waits_preserve_scoped_debts_and_foreign_gaps() -> TestResult {
    for (implementation, expected) in [
        ("impl<'a>A<'a>for u8{type Out=u8;}", true),
        ("impl A<'static>for u8{type Out=u8;}", false),
        ("impl<'a:'static>A<'a>for u8{type Out=u8;}", false),
    ] {
        let (report, mut facts, goal) = fixture(implementation)?;
        let mut delayed = header_tests::delay(
            &mut facts,
            &[
                tests::definition(&report, "A")?,
                tests::definition(&report, "Accept")?,
            ],
        )?;
        let assumptions = Assumptions::default();
        let (mut context, goal) = tests::instantiate(&mut facts, goal)?;
        let mut search = Search::new(
            tests::caller(&assumptions, None),
            Availability::Declarations,
        );
        let frame = Frame::new(&facts, &mut context, &goal, 1)?;
        let mut progress = search.begin(&mut facts, frame)?;
        let mut gaps = Vec::new();
        while let Progress::Pending(crate::sema::traits::selection::Dependency::Header(owner)) =
            progress
        {
            header_tests::stable_poll(&mut search, &mut facts, owner)?;
            let end = search.query.allocations.end();
            gaps.push(end..end + 3);
            search.resume_at(end + 3)?;
            let index = delayed
                .iter()
                .position(|header| header.owner == owner)
                .ok_or("unexpected callable header wait")?;
            header_tests::publish(&mut facts, delayed.swap_remove(index))?;
            progress = search.poll(&mut facts)?;
        }
        assert!(delayed.is_empty());
        assert_eq!(gaps.len(), 2);
        let Progress::Ready(Decision::Proven(root)) = progress else {
            return Err("callable normalization failed to resume".into());
        };
        let proof = search.evidence(&facts, root)?.proof;
        assert!(proof.scope_count() > 0);
        for id in gaps.into_iter().flatten() {
            assert!(!proof.owns_region(RegionId(id)), "foreign allocation {id}");
        }
        assert_eq!(valid(&facts, &proof, 0)?, expected, "{implementation}");
    }
    Ok(())
}
