//! Contracting real selected suffixes cannot discharge their retained lifetime proof.
use super::{Decision, Frame, Search, size_evidence_tests, tests};
use crate::{
    sema::traits::{assumptions::Assumptions, implementations::Availability},
    types::{
        GenericArgs, ItemId, Kind, Primitive, RegionId, TypeId, arguments::Argument,
        projection::Application, substitute::Alias,
    },
};

#[test]
fn actual_deref_chain_keeps_the_whole_static_requirement() -> Result<(), Box<dyn std::error::Error>>
{
    let source =
        crate::test_source::fixture("pa31/tests/spec/dereference-source/receiver-chain-static.rs")?;
    for (region, valid) in [(RegionId::STATIC, true), (RegionId(10), false)] {
        let report = crate::test_source::source_provider(&source, "chain_lifetimes")?;
        let mut facts = crate::sema::declarations::analyze(&report)?;
        let scalar = TypeId::primitive(Primitive::U8);
        let mut subject = facts.types.intern(Kind::Reference {
            mutable: false,
            region,
            referent: scalar,
        })?;
        let pin = ItemId(tests::definition(&report, "Pin")?.0);
        for _ in 0..2 {
            let arguments = facts.types.intern_arguments(&[Argument::Type(subject)])?;
            subject = facts.types.intern(Kind::Adt(pin, arguments))?;
        }
        let predicate = tests::predicate(
            &mut facts,
            tests::definition(&report, "Deref")?,
            subject,
            GenericArgs::EMPTY,
        )?;
        let (application, _) = predicate.bound.trait_parts().ok_or("Deref application")?;
        let alias = Alias::Associated(Application {
            item: ItemId(tests::definition(&report, "Target")?.0),
            subject,
            trait_application: application,
            arguments: GenericArgs::EMPTY,
        });
        let (mut context, goal) = tests::instantiate(&mut facts, predicate)?;
        let assumptions = Assumptions::default();
        let mut caller = tests::caller(&assumptions, None);
        caller.first_region = 100;
        let mut search = Search::new(caller, Availability::Validated);
        let frame = Frame::new(&facts, &mut context, &goal, 100)?;
        let Decision::Proven(root) = search.run(&mut facts, frame)? else {
            return Err("missing actual Deref parent proof".into());
        };
        let (chain, work) = search.selected_chain(&facts, root)?;
        let chain = chain.ok_or("missing selected Deref suffixes")?;
        assert!(work >= 3);
        let (result, steps) = chain.resolve(&mut facts, alias)?;
        assert_eq!(result, Some(scalar));
        assert_eq!(steps, 3);
        let evidence = search.evidence(&facts, root, Some(chain))?;
        assert_eq!(size_evidence_tests::valid(&facts, &evidence.proof)?, valid);
    }
    Ok(())
}
