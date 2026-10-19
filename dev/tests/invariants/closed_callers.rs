//! Opening a request preserves a closed caller's shared permissions and local binders.
use super::{Availability, Declarations, Environment, Predicate, prepare};
use crate::sema::traits::assumptions::Selection;
use crate::types::{
    ItemId, RegionId, Store, TypeId,
    arguments::Argument,
    infer::{self, Instantiation},
    parameters::{
        clauses::{Clause, Quantified},
        predicates::Bound,
    },
    traits::Application,
};
use std::sync::Arc;

struct NoFiles;
impl crate::resolve::load::Sources for NoFiles {
    fn read(
        &mut self,
        _: &std::path::Path,
    ) -> Result<Option<crate::resolve::load::InputFile>, String> {
        Ok(None)
    }
}

fn empty_declarations() -> Result<Declarations, &'static str> {
    let input = crate::resolve::load::InputFile {
        identity: "opening.rs".into(),
        path: "opening.rs".into(),
        source: crate::support::source::SourceFile::new(Vec::new())
            .map_err(|_| "empty test source failed")?,
    };
    let expanded = crate::resolve::expansion::analyze(
        input,
        &mut NoFiles,
        &crate::expand::config::Config::default(),
    )
    .map_err(|_| "empty test crate expansion failed")?;
    let report = crate::resolve::analyze_expanded(expanded)
        .map_err(|_| "empty test crate resolution failed")?;
    Declarations::new(&report).map_err(|_| "empty test declarations failed")
}

fn predicate(store: &mut Store, item: u32, region: RegionId) -> Result<Predicate, &'static str> {
    let arguments = store.intern_arguments(&[Argument::Lifetime(region)])?;
    let application = store.intern_trait_application(Application {
        item: ItemId(item),
        arguments,
    })?;
    Ok(Predicate {
        ty: TypeId::UNIT,
        bound: Bound::Trait(application),
    })
}

fn assert_absent(
    facts: &mut Declarations,
    state: &mut super::State,
    predicate: Predicate,
) -> Result<(), infer::Error> {
    let mut substitution = Instantiation::new(&mut state.context, &facts.types, &[])?;
    let goal =
        state
            .context
            .instantiate_predicate(&mut facts.types, predicate, &mut substitution)?;
    assert!(matches!(
        state.assumptions.select_scoped(
            &mut facts.types,
            &mut state.context,
            &goal,
            state.obligations.regions.end,
        )?,
        Selection::Absent
    ));
    Ok(())
}

#[test]
fn open_requests_share_closed_quantified_environments_and_keep_private_aliases()
-> Result<(), &'static str> {
    let mut opening_work = None;
    for width in [16, 64, 256, 1024] {
        let mut facts = empty_declarations()?;
        let mut clauses = Vec::new();
        for item in 0..width {
            let predicate = predicate(&mut facts.types, item, RegionId::bound(0, 0)?)?;
            clauses.push(Clause::ForAll(facts.types.intern_quantified(
                Quantified {
                    bound_regions: 1,
                    predicate,
                },
            )?));
        }
        let environment =
            Arc::new(Environment::new(&facts.types, &[])?.with_clauses(&facts.types, &clauses)?);
        assert!(!environment.requires_region_mapping());
        let surrounding = RegionId::bound(0, 0)?;
        let requested = predicate(&mut facts.types, width, surrounding)?;
        for (request, first) in [100, 200, 300].into_iter().enumerate() {
            let (mut state, opened) = prepare(
                &mut facts,
                Some(environment.clone()),
                requested,
                first,
                false,
                Availability::Declarations,
            )?;
            assert!(Arc::ptr_eq(
                state.environment.as_ref().ok_or("missing caller")?,
                &environment
            ));
            assert_eq!(opened, predicate(&mut facts.types, width, RegionId(first))?);
            assert_eq!(state.obligations.regions, first..first + 1);
            assert_eq!(
                state.obligations.equalities,
                [(RegionId(first), surrounding)]
            );
            // Index construction is charged once, including all caller clauses.
            // Later queries only borrow that index and perform their own opening.
            let construction = if request == 0 { width as usize } else { 0 };
            let query_work = state.context.steps - construction;
            assert_eq!(*opening_work.get_or_insert(query_work), query_work);
            // The request cannot become a caller assumption merely by opening it.
            assert_absent(&mut facts, &mut state, opened)
                .map_err(|_| "invalid scoped assumption lookup")?;
        }
        assert_eq!(environment.trait_clauses().collect::<Vec<_>>(), clauses);
    }
    Ok(())
}

#[test]
fn captured_environments_still_share_the_requests_mapping_without_mutating_the_caller()
-> Result<(), &'static str> {
    let mut facts = empty_declarations()?;
    let surrounding = RegionId::bound(0, 0)?;
    let original = predicate(&mut facts.types, 1, surrounding)?;
    let environment =
        Arc::new(Environment::new(&facts.types, &[])?.with_predicates(&facts.types, &[original])?);
    assert!(environment.requires_region_mapping());
    let (state, opened) = prepare(
        &mut facts,
        Some(environment.clone()),
        original,
        100,
        false,
        Availability::Declarations,
    )?;
    let mapped = state
        .environment
        .as_ref()
        .ok_or("missing mapped environment")?;
    assert!(!Arc::ptr_eq(mapped, &environment));
    assert_eq!(opened, predicate(&mut facts.types, 1, RegionId(100))?);
    assert_eq!(
        mapped.trait_clauses().collect::<Vec<_>>(),
        [Clause::Predicate(opened)]
    );
    assert_eq!(
        environment.trait_clauses().collect::<Vec<_>>(),
        [Clause::Predicate(original)]
    );
    assert_eq!(state.obligations.regions, 100..101);
    assert_eq!(state.obligations.equalities, [(RegionId(100), surrounding)]);
    Ok(())
}
