//! Real fresh consumer traits retain the shared remapper's move-only input seal.
use super::*;
use crate::{
    expand::config::Config,
    resolve::{
        bindings::{Context, consumer_policy::ProviderAdmission},
        crates::{ConsumerInput, ConsumerReservation, unpaid_provided::Inputs},
        expansion::Report,
        load::{InputFile, Sources},
    },
    sema::traits::inherited_inputs::HeaderInputs,
    support::{bindings::Mode, crates::CrateIdentity, source::SourceFile},
    unpaid_cases::Result,
};
use std::path::Path;

struct NoFiles;
impl Sources for NoFiles {
    fn read(&mut self, _: &Path) -> std::result::Result<Option<InputFile>, String> {
        Ok(None)
    }
}
fn fresh(work: &mut ReservationWork) -> Result<(Report, ConsumerReservation, Declarations)> {
    let mut actual = crate::unpaid_cases::trait_extended(inherited_tests::HOLDER)?;
    let provider = actual.inventory.artifacts()[0].identity().clone();
    let producer_arena = actual.facts.types.arena_identity();
    drop(actual.facts);
    let context = Context {
        crate_name: Some("consumer".into()),
        current: Some(CrateIdentity::new("consumer", "canonical_inputs")?),
        dependencies: Some(Arc::clone(&actual.inventory)),
        mode: Mode::SourceClient { provider },
    };
    let inputs = Inputs::reserve(actual.report, actual.inventory, 0, work)?;
    let admission = ProviderAdmission::from_reserved(inputs, actual.contracts, work)?;
    let input = ConsumerInput::new(
        InputFile {
            identity: "canonical-inputs.rs".into(),
            path: "canonical-inputs.rs".into(),
            source: SourceFile::new(b"#![feature(no_core)]#![no_core]fn main(){}".to_vec())?,
        },
        &mut NoFiles,
        &Config::default(),
        &context,
        admission,
        work,
    )?;
    let (report, reservation) = input.into_parts();
    let facts = Declarations::new_staged(&report, work)?;
    assert!(!facts.types.has_identity(&producer_arena));
    Ok((report, reservation, facts))
}

fn prepare_scope(
    view: RecipeView<'_>,
    facts: &mut Declarations,
    work: &mut ReservationWork,
) -> Result<(State, Owner, usize)> {
    let rows = view.rows()?;
    let holder = rows
        .traits
        .iter()
        .find(|header| {
            rows.declarations[header.declaration.0 as usize]
                .name
                .as_deref()
                == Some("Holder")
        })
        .ok_or("actual retained nonrole Holder")?;
    prepare_owner_scope(view, facts, holder.scope, work)
}
fn prepare_owner_scope(
    view: RecipeView<'_>,
    facts: &mut Declarations,
    declaration: DeclarationRef,
    work: &mut ReservationWork,
) -> Result<(State, Owner, usize)> {
    let rows = view.rows()?;
    let mut state = State::new(&view, &mut facts.types, work)?;
    let table = Prepared::new(&view, work)?;
    let path = formal_path::path(rows, &table.scopes, declaration, work)?;
    let mut owner = Owner::new(rows, work)?;
    let mut port = ports::Inputs {
        port: Reserved::new(view, &state.maps, &facts.types, work)?,
        cursor: &mut state.cursor,
    };
    let mut parent = None;
    for index in path.indices.into_iter().rev() {
        owner.scope(rows, &mut port, facts, index, parent, work)?;
        parent = Some(index);
    }
    Ok((state, owner, parent.ok_or("actual Holder scope")?))
}

#[test]
fn shared_trait_input_producer_seals_actual_fresh_catalog_and_canonical_prefix() -> Result {
    let mut work = ReservationWork::default();
    let (report, reservation, mut facts) = fresh(&mut work)?;
    let view = reservation.recipe_view(&report, &mut work)?;
    let rows = view.rows()?;
    assert_eq!(rows.roles.len(), 9);
    let (mut state, mut owner, index) = prepare_scope(view, &mut facts, &mut work)?;
    foreign_warm_inputs(
        &report,
        &reservation,
        &mut facts,
        &mut state,
        &mut owner,
        index,
        &mut work,
    )?;
    let scope = Rc::get_mut(&mut owner.scopes).ok_or("exclusive actual scope table")?[index]
        .take()
        .ok_or("actual prepared trait inputs")?;
    assert_eq!(scope.parameters.len(), 2, "genuine Self/V remain complete");
    let ScopeInputs::Trait { prepared, .. } = scope.inputs else {
        return Err("actual trait inputs lost their canonical seal".into());
    };
    let prepared = prepared
        .into_inner()
        .ok_or("actual untransferred trait producer")?;
    let all = Arc::clone(prepared.all());
    let subjects = prepared.subjects().to_vec();
    let holder = rows
        .traits
        .iter()
        .find(|header| header.scope == scope.raw.owner)
        .ok_or("actual Holder header")?;
    let view = reservation.recipe_view(&report, &mut work)?;
    let definition = view.definition(holder.declaration, &mut work)?;
    let other = view.definition(rows.traits[0].declaration, &mut work)?;
    assert_ne!(definition, other);
    assert_eq!(all.len(), holder.inputs.slice(&rows.clauses)?.len());
    assert_eq!(subjects.len(), holder.canonical_inputs as usize);
    let inputs = HeaderInputs::imported(prepared);
    inputs.require_catalog(reservation.catalog(), definition)?;
    assert!(
        inputs
            .require_catalog(reservation.catalog(), other)
            .is_err()
    );
    assert_eq!(inputs.view().len(), subjects.len());
    for (index, actual) in inputs.view().iter().enumerate() {
        assert_eq!(actual.source, all[index].source);
        assert_eq!(actual.value, all[index].value);
        assert_eq!(inputs.is_self(index, actual)?, subjects[index]);
    }
    let (_, foreign, _) = fresh(&mut ReservationWork::default())?;
    assert!(
        inputs
            .require_catalog(foreign.catalog(), definition)
            .is_err()
    );
    assert!(facts.definitions.iter().all(Option::is_none));
    assert!(facts.functions.is_empty());
    assert!(facts.generics.traits.is_none());
    actual_const_leaf_inputs()?;
    eprintln!(
        "shared-input-layout scope={} inputs={} prepared={} owner={} session={} state={} retained={} peak={} steps={}",
        size_of::<Scope>(),
        size_of::<ScopeInputs>(),
        size_of::<predicate_remap::Prepared>(),
        size_of::<Owner>(),
        size_of::<Session>(),
        size_of::<State>(),
        work.retained,
        work.peak,
        work.steps
    );
    Ok(())
}

fn actual_const_leaf_inputs() -> Result {
    let mut work = ReservationWork::default();
    let (report, reservation, mut facts) = fresh(&mut work)?;
    let view = reservation.recipe_view(&report, &mut work)?;
    let rows = view.rows()?;
    let header = rows
        .functions
        .iter()
        .find(|header| {
            rows.declarations[header.declaration.0 as usize]
                .name
                .as_deref()
                == Some("clone_from")
        })
        .ok_or("actual const trait default member")?;
    let (_, owner, index) = prepare_owner_scope(view, &mut facts, header.scope, &mut work)?;
    let scope = owner.scopes[index]
        .get()
        .map(Box::as_ref)
        .ok_or("actual const leaf scope")?;
    let mut inputs = scope.inputs.all().to_vec();
    let input = inputs
        .iter_mut()
        .find(|input| {
            matches!(
                input.value,
                Clause::Predicate(Predicate {
                    bound: Bound::MaybeConstTrait(_) | Bound::ConstTrait(_),
                    ..
                })
            )
        })
        .ok_or("actual effect-qualified source input")?;
    let Clause::Predicate(predicate) = &mut input.value else {
        return Err("actual ordinary const leaf predicate".into());
    };
    let (application, effect) = predicate.bound.trait_parts().ok_or("actual trait effect")?;
    assert_ne!(effect, crate::support::constness::Constness::Ordinary);
    predicate.bound = Bound::Trait(application);
    let attempted = work.steps;
    assert!(
        originals::require_leaf_inputs(rows, scope.raw, &inputs, &owner.applications, &mut work)
            .is_err()
    );
    assert!(work.steps > attempted);
    assert!(facts.definitions.iter().all(Option::is_none));
    assert!(facts.generics.traits.is_none());
    assert!(
        owner.bodies.iter().all(Option::is_none),
        "scope construction cannot imply body completion"
    );
    Ok(())
}

fn foreign_warm_inputs(
    report: &Report,
    reservation: &ConsumerReservation,
    facts: &mut Declarations,
    state: &mut State,
    owner: &mut Owner,
    index: usize,
    work: &mut ReservationWork,
) -> Result {
    let mut scratch = ReservationWork::default();
    let (other_report, other_reservation, mut other_facts) = fresh(&mut scratch)?;
    let (_, mut other, other_index) = prepare_scope(
        other_reservation.recipe_view(&other_report, &mut scratch)?,
        &mut other_facts,
        &mut scratch,
    )?;
    let other_inputs = Rc::get_mut(&mut other.scopes).ok_or("exclusive foreign scope table")?
        [other_index]
        .take()
        .ok_or("actual foreign scope")?
        .inputs;
    let original = std::mem::replace(
        &mut Rc::get_mut(&mut owner.scopes).ok_or("exclusive actual scope table")?[index]
            .get_mut()
            .ok_or("actual scope")?
            .inputs,
        other_inputs,
    );
    let declared_parent = owner.scopes[index]
        .get()
        .map(Box::as_ref)
        .ok_or("actual scope")?
        .raw
        .parent;
    let parent = declared_parent
        .map(|declaration| {
            owner
                .scopes
                .iter()
                .position(|scope| {
                    scope
                        .get()
                        .map(Box::as_ref)
                        .is_some_and(|scope| scope.raw.owner == declaration)
                })
                .ok_or("actual parent scope")
        })
        .transpose()?;
    let view = reservation.recipe_view(report, work)?;
    let rows = view.rows()?;
    let mut port = ports::Inputs {
        port: Reserved::new(view, &state.maps, &facts.types, work)?,
        cursor: &mut state.cursor,
    };
    let before = work.steps;
    assert!(
        owner
            .scope(rows, &mut port, facts, index, parent, work)
            .is_err()
    );
    assert!(work.steps > before);
    Rc::get_mut(&mut owner.scopes).ok_or("exclusive actual scope table")?[index]
        .get_mut()
        .ok_or("actual scope")?
        .inputs = original;
    owner.scope(rows, &mut port, facts, index, parent, work)?;
    Ok(())
}
