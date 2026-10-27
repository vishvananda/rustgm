//! Formation cache entries are private WF prefixes, never trait authority.
use super::*;
use crate::{
    sema::traits::{assumptions::Assumptions, implementations::Availability},
    types::{GenericArgs, ItemId, Kind, arguments::Argument, lifetime_proof::ScopeId},
};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
#[test]
fn active_completed_and_binder_depth_formation_entries_remain_distinct() -> Result {
    let source =
        crate::test_source::fixture("pa31/tests/spec/source-freeze/nominal-no-override.rs")?;
    let report = crate::test_source::source_provider(&source, "formation_cache")?;
    let mut facts = crate::sema::declarations::analyze(&report)?;
    let n = facts.types.intern(Kind::Adt(
        ItemId(tests::definition(&report, "N")?.0),
        GenericArgs::EMPTY,
    ))?;
    let arguments = facts.types.intern_arguments(&[Argument::Type(n)])?;
    let ty = facts.types.intern(Kind::Adt(
        ItemId(tests::definition(&report, "W")?.0),
        arguments,
    ))?;
    let bound = freezing::bound(&mut facts, crate::types::auto_traits::AutoTrait::Freeze)?;
    let (mut context, goal) = tests::instantiate(&mut facts, Predicate { ty, bound })?;
    let assumptions = Assumptions::default();
    let mut search = Search::new(tests::caller(&assumptions, None), Availability::Validated);
    let mut frame = Frame::new(&facts, &mut context, &goal, 1)?;
    assert!(matches!(
        search.advance(&mut facts, &mut frame)?,
        Action::Continue
    ));
    assert!(!frame.formation_cacheable());
    assert!(frame.key.is_some());
    assert!(
        search
            .complete_nominal_formation(&mut facts, &mut frame, Decision::Proven(0))
            .is_err()
    );
    assert!(matches!(
        search
            .query
            .families
            .get(&member_conditions::Key::Nominal(ty, 0, ScopeId::ROOT)),
        Some(member_conditions::Entry::Active)
    ));
    let Decision::Proven(root) = search.run(&mut facts, frame)? else {
        return Err("nominal proof".into());
    };
    let Some(member_conditions::Entry::Completed(prefix)) = search
        .query
        .families
        .get(&member_conditions::Key::Nominal(ty, 0, ScopeId::ROOT))
    else {
        return Err("completed formation".into());
    };
    let prefix = *prefix;
    assert_ne!(root, prefix);
    assert!(search.query.nodes[prefix].source.is_none());
    let mut parent = Frame::new(&facts, &mut context, &goal, 1)?;
    assert!(!search.prepare_nominal_formation(&mut facts, &mut parent)?);
    assert!(parent.formation_cacheable());
    search.query.nodes[prefix].debt_free = false;
    assert!(
        search
            .complete_nominal_formation(&mut facts, &mut parent, Decision::Absent)
            .is_err()
    );
    search.query.nodes[prefix].debt_free = true;
    assert!(matches!(
        search.complete_nominal_formation(&mut facts, &mut parent, Decision::Absent)?,
        Decision::Absent
    ));
    let mut helper =
        search.nominal_formation_child_for_test(&mut facts, &mut context, &goal, ty, 0)?;
    let before = search.query.nodes.len();
    assert!(
        matches!(search.advance(&mut facts,&mut helper)?,Action::Complete(Decision::Proven(node)) if node==prefix)
    );
    assert!(!helper.formation_cacheable());
    assert_eq!(before, search.query.nodes.len());
    let _other = search.nominal_formation_child_for_test(&mut facts, &mut context, &goal, ty, 1)?;
    assert!(matches!(
        search
            .query
            .families
            .get(&member_conditions::Key::Nominal(ty, 1, ScopeId::ROOT)),
        Some(member_conditions::Entry::Active)
    ));
    assert!(search.evidence(&facts, prefix).is_err());
    Ok(())
}
