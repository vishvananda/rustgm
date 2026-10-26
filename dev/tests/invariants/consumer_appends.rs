//! The actual consumer table receives selected templates without header proof.
use super::*;
use crate::{
    support::crates::owner_catalog::DeclarationRef,
    types::{ItemId, callable::Templates},
};

fn start(source: &str, work: &mut ReservationWork) -> Result<Box<HeaderDraft>> {
    let actual = fixture(source)?;
    construct(
        input(MAIN)?,
        &mut NoFiles,
        &Config::default(),
        &actual.context,
        actual.admission,
        work,
    )?
    .selected_headers(work)
    .map_err(Into::into)
}
fn named(stage: &HeaderDraft, name: &str, work: &mut ReservationWork) -> Result<DeclarationRef> {
    let authority = &stage.session.authority;
    let view = authority.reservation.recipe_view(&authority.report, work)?;
    let rows = view.rows()?;
    rows.functions
        .iter()
        .find(|header| {
            rows.declarations[header.declaration.0 as usize]
                .name
                .as_deref()
                == Some(name)
        })
        .map(|header| header.declaration)
        .ok_or_else(|| "actual selected function is missing".into())
}
fn ready(progress: HeaderProgress) -> Result<Box<HeaderDraft>> {
    let (stage, result) = progress;
    if result != Demand::Ready(()) {
        return Err("actual selected callable is unexpectedly blocked".into());
    }
    Ok(stage)
}
fn registered(
    stage: &HeaderDraft,
    owner: DeclarationRef,
    work: &mut ReservationWork,
) -> Result<usize> {
    let authority = &stage.session.authority;
    let view = authority.reservation.recipe_view(&authority.report, work)?;
    let item = ItemId(view.definition(owner, work)?.0);
    let (template, formals) = authority
        .facts
        .lifetimes
        .callables
        .registration(item)
        .ok_or("selected registration is absent from actual consumer facts")?;
    assert_eq!(template.parameters, 0);
    assert!(authority.facts.types.kind(template.signature).is_some());
    Ok(formals.len())
}

#[test]
fn selected_templates_append_full_family_to_actual_consumer_table() -> Result {
    let mut source = HOLDER.to_owned();
    source.push_str("pub fn plain(value:u8)->u8{value}pub fn unused<T>(value:u8)->u8{value}");
    let mut work = ReservationWork::default();
    let mut stage = start(&source, &mut work)?;
    for (name, count) in [
        ("clone", 1),
        ("clone_from", 1),
        ("identity", 1),
        ("carry", 3),
        ("borrow", 2),
        ("plain", 0),
        ("unused", 1),
    ] {
        let owner = named(&stage, name, &mut work)?;
        stage = ready(stage.callable_template(owner, &mut work)?)?;
        assert_eq!(registered(&stage, owner, &mut work)?, count, "{name}");
    }
    let authority = &stage.session.authority;
    assert!(authority.facts.definitions.iter().all(Option::is_none));
    assert!(authority.facts.functions.is_empty());
    let catalog = authority.facts.source_bindings.as_deref().ok_or("roles")?;
    for role in [LangItem::Clone, LangItem::Copy, LangItem::Sized] {
        assert!(catalog.validated_role(role).is_err());
        let definition = catalog
            .role(role)?
            .ok_or("actual retained role")?
            .definition;
        assert!(authority.facts.associated.owner(definition).is_none());
        assert!(authority.facts.function_header(definition)?.is_none());
        assert!(
            authority
                .facts
                .header_attribute_contracts(&authority.report, definition)
                .is_err()
        );
    }
    eprintln!(
        "consumer-selected-layout draft={} session={} cursor={} view={} retained={} peak={} steps={}",
        size_of::<HeaderDraft>(),
        size_of::<ProofSession>(),
        size_of::<SelectedHeaders>(),
        size_of::<crate::resolve::crates::unpaid_provided::RecipeView<'_>>(),
        work.retained,
        work.peak,
        work.steps
    );
    Ok(())
}

#[test]
fn selected_templates_reject_replaced_warm_authority_and_registration() -> Result {
    for opponent in 0..4 {
        let mut work = ReservationWork::default();
        let mut first = start(HOLDER, &mut work)?;
        let mut second = start(HOLDER, &mut ReservationWork::default())?;
        let owner = named(&first, "carry", &mut work)?;
        first = ready(first.callable_template(owner, &mut work)?)?;
        match opponent {
            0 => first.session.authority.facts.lifetimes.callables = Templates::default(),
            1 => std::mem::swap(
                &mut first.session.authority.facts.types,
                &mut second.session.authority.facts.types,
            ),
            2 => std::mem::swap(
                &mut first.session.authority.report,
                &mut second.session.authority.report,
            ),
            _ => std::mem::swap(
                &mut first.session.authority.reservation,
                &mut second.session.authority.reservation,
            ),
        }
        let before = work.steps;
        assert!(first.callable_template(owner, &mut work).is_err());
        assert!(work.steps > before);
    }
    Ok(())
}

fn warm(count: usize) -> Result<usize> {
    let mut source = HOLDER.to_owned();
    for index in 0..count {
        write!(source, "pub fn other_{index}<T>(value:T)->T{{value}}")?;
    }
    let mut work = ReservationWork::default();
    let mut stage = start(&source, &mut work)?;
    let owner = named(&stage, "carry", &mut work)?;
    stage = ready(stage.callable_template(owner, &mut work)?)?;
    let before = (
        stage.session.authority.facts.types.interned_extent(),
        stage
            .session
            .authority
            .facts
            .lifetimes
            .callables
            .allocation(),
        work.retained,
        work.peak,
    );
    let steps = work.steps;
    for _ in 0..1000 {
        stage = ready(stage.callable_template(owner, &mut work)?)?;
    }
    let delta = work.steps - steps;
    assert_eq!(
        before.0,
        stage.session.authority.facts.types.interned_extent()
    );
    let allocation = stage
        .session
        .authority
        .facts
        .lifetimes
        .callables
        .allocation();
    assert_eq!(
        (before.1.bytes, before.1.visits),
        (allocation.bytes, allocation.visits)
    );
    assert_eq!((before.2, before.3), (work.retained, work.peak));
    eprintln!(
        "consumer-selected unrelated={count} warm1000={delta} retained={} peak={}",
        work.retained, work.peak
    );
    Ok(delta)
}

#[test]
fn selected_templates_bound_warm_work_and_failed_preflight() -> Result {
    let mut previous = None;
    for count in [1, 8, 32] {
        let steps = warm(count)?;
        if let Some(previous) = previous {
            assert_eq!(steps, previous);
        }
        previous = Some(steps);
    }
    let mut work = ReservationWork::default();
    let stage = start(HOLDER, &mut work)?;
    let owner = named(&stage, "carry", &mut work)?;
    work.retained = crate::support::crates::MAX_BYTES;
    let before = work.steps;
    assert!(stage.callable_template(owner, &mut work).is_err());
    assert!(work.steps > before);
    assert!(work.peak > crate::support::crates::MAX_BYTES);
    Ok(())
}
