//! Real private assumptions retain declaration provenance without caller proof.
use super::*;
use crate::unpaid_cases::{Result, actual, extended};
use std::fmt::Write as _;

fn named(inventory: &crate::support::crates::Dependencies, name: &str) -> Result<DeclarationRef> {
    let rows = inventory.artifacts()[0]
        .source_descriptor()
        .ok_or("actual descriptor")?
        .rows();
    rows.functions
        .iter()
        .find(|row| {
            rows.declarations[row.declaration.0 as usize]
                .name
                .as_deref()
                == Some(name)
        })
        .map(|row| row.declaration)
        .ok_or_else(|| "actual named original".into())
}
fn start(
    source: &str,
    name: &str,
    work: &mut ReservationWork,
) -> Result<(Box<Session>, DeclarationRef)> {
    let (report, facts, inventory) = extended(source)?;
    let owner = named(&inventory, name)?;
    Ok((
        Session::prepare(report, facts, inventory, 0, work)?.boxed(work)?,
        owner,
    ))
}
fn ready(progress: Progress) -> Result<Box<Session>> {
    let (session, status) = progress;
    if status != Demand::Ready(()) {
        return Err("actual zero-domain assumptions unexpectedly blocked".into());
    }
    Ok(session)
}
fn environments(session: &Session) -> Result<&Owner> {
    session
        .recipes
        .as_ref()
        .and_then(|state| state.callables.as_ref())
        .and_then(|prepared| prepared.environments.as_deref())
        .ok_or_else(|| "actual private environment owner".into())
}
#[test]
fn private_defining_environments_use_real_zero_domain_assumptions() -> Result {
    let mut work = ReservationWork::default();
    let (session, monomorphic) = start(
        "\nfn bounded(x:u8)->u8 where u8:Copy{x}\nfn unused<T>(x:u8)->u8{x}\nfn plain(x:u8)->u8{x}",
        "bounded",
        &mut work,
    )?;
    let old = session.old_definitions;
    let mut session = ready(session.defining_environment(monomorphic, &mut work)?)?;
    let unused = named(&session.inventory, "unused")?;
    session = ready(session.defining_environment(unused, &mut work)?)?;
    let owner = environments(&session)?;
    let mut counts = Vec::new();
    for body in owner.bodies.iter().flatten() {
        let Body::Ready {
            expected,
            scope,
            environment,
        } = body.as_ref()
        else {
            return Err("actual ready environment record".into());
        };
        let declaration = owner.scopes[*scope]
            .as_deref()
            .ok_or("actual declaration environment")?;
        assert!(Arc::ptr_eq(environment, &declaration.environment));
        assert_eq!(declaration.parameters.len(), expected.formals);
        assert!(
            declaration
                .inputs
                .iter()
                .all(|input| matches!(input.value, Clause::Predicate(_)))
        );
        counts.push(expected.formals);
    }
    counts.sort_unstable();
    assert_eq!(counts, [0, 1]);
    let plain = named(&session.inventory, "plain")?;
    let (next, status) = session.defining_environment(plain, &mut work)?;
    session = next;
    assert!(matches!(
        status,
        Demand::Blocked(Dependency::OwnedBodyDomain(_))
    ));
    assert!(session.facts.definitions[old..].iter().all(Option::is_none));
    session.require(&mut work)?;
    let default = named(&session.inventory, "clone_from")?;
    let (session, status) = session.defining_environment(default, &mut work)?;
    assert!(matches!(
        status,
        Demand::Blocked(Dependency::OwnedBodyDomain(_))
    ));
    let rows = session.inventory.artifacts()[0]
        .source_descriptor()
        .ok_or("actual full family")?
        .rows();
    assert_eq!(rows.roles.len(), 9);
    assert!(
        rows.functions
            .iter()
            .any(|header| header.body == s::FunctionBody::Absent)
    );
    let required = rows
        .functions
        .iter()
        .find(|header| header.body == s::FunctionBody::Absent)
        .ok_or("actual required method")?
        .declaration;
    assert!(session.defining_environment(required, &mut work).is_err());
    // The real general-trait producer is a named prerequisite, not a fake role.
    let error = extended("\ntrait Holder<V>{fn carry<U>(value:U)->U{value}}")
        .err()
        .ok_or("actual nonrole trait unexpectedly bypassed its authored producer")?;
    assert!(
        error
            .to_string()
            .contains("source trait graph contains an unsupported unvalidated declaration")
    );
    Ok(())
}
fn change_owner(
    mut session: Box<Session>,
    owner: DeclarationRef,
    opponent: usize,
    work: &mut ReservationWork,
) -> Result<Box<Session>> {
    match opponent {
        0 => {
            let (_, _, foreign) = actual(true)?;
            session.inventory = foreign;
        }
        1 => {
            session.facts.types = crate::types::Store::default();
        }
        2 => {
            let prepared = session
                .recipes
                .as_mut()
                .and_then(|state| state.callables.as_mut())
                .ok_or("actual template owner")?;
            prepared.scopes[owner.0 as usize] = None;
        }
        3 => {
            work.steps = crate::support::crates::MAX_COMPONENTS;
        }
        4 => {
            let actual = session.inputs.recipe_view(&session.inventory, 0, work)?;
            assert!(actual.input_site(owner, u32::MAX, work).is_err());
            let prepared = session
                .recipes
                .as_mut()
                .and_then(|state| state.callables.as_mut())
                .ok_or("actual template owner")?;
            let env = prepared
                .environments
                .as_mut()
                .ok_or("actual environment owner")?;
            let record = env
                .bodies
                .iter_mut()
                .flatten()
                .next()
                .ok_or("actual defining record")?;
            let Body::Ready { scope, .. } = record.as_mut() else {
                return Err("actual ready record".into());
            };
            *scope = usize::MAX;
        }
        _ => {
            let other = named(&session.inventory, "identity")?;
            session = ready(session.defining_environment(other, work)?)?;
            let prepared = session
                .recipes
                .as_mut()
                .and_then(|state| state.callables.as_mut())
                .ok_or("actual private template owner")?;
            let env = prepared
                .environments
                .as_mut()
                .ok_or("actual environment owner")?;
            let foreign = env
                .bodies
                .iter()
                .flatten()
                .find_map(|body| match body.as_ref() {
                    Body::Ready {
                        expected,
                        environment,
                        ..
                    } if expected.header.declaration == other => Some(Arc::clone(environment)),
                    _ => None,
                })
                .ok_or("other actual defining environment")?;
            let selected = env
                .bodies
                .iter()
                .flatten()
                .find_map(|body| match body.as_ref() {
                    Body::Ready {
                        expected, scope, ..
                    } if expected.header.declaration == owner => Some(*scope),
                    _ => None,
                })
                .ok_or("selected actual declaration environment")?;
            env.scopes[selected]
                .as_mut()
                .ok_or("actual selected scope")?
                .environment = foreign;
        }
    }
    Ok(session)
}
#[test]
fn private_defining_environments_reject_changed_owners_and_failed_budgets() -> Result {
    for opponent in 0..6 {
        let mut work = ReservationWork::default();
        let (session, owner) = start("\nfn unused<T>(x:u8)->u8{x}", "unused", &mut work)?;
        let mut session = ready(session.defining_environment(owner, &mut work)?)?;
        session = change_owner(session, owner, opponent, &mut work)?;
        let before = work.steps;
        assert!(session.defining_environment(owner, &mut work).is_err());
        assert!(work.steps > before);
    }
    let mut work = ReservationWork::default();
    let (session, owner) = start("\nfn unused<T>(x:u8)->u8{x}", "unused", &mut work)?;
    let session = ready(session.callable_template(owner, &mut work)?)?;
    work.retained = crate::support::crates::MAX_BYTES;
    assert!(session.defining_environment(owner, &mut work).is_err());
    assert!(work.peak > crate::support::crates::MAX_BYTES);
    let mut work = ReservationWork::default();
    let (session, owner) = start("\nfn regional<T:'static>(x:T)->T{x}", "regional", &mut work)?;
    let (session, status) = session.defining_environment(owner, &mut work)?;
    assert!(matches!(
        status,
        Demand::Blocked(Dependency::OwnedBodyDomain(_))
    ));
    assert!(
        session
            .recipes
            .as_ref()
            .and_then(|state| state.callables.as_ref())
            .is_some_and(|p| p.environments.is_none())
    );
    Ok(())
}
#[test]
fn private_defining_environments_bound_selected_and_warm_work() -> Result {
    eprintln!(
        "private defining layouts: owner={} scope={} body={} templates={}",
        size_of::<Owner>(),
        size_of::<Scope>(),
        size_of::<Body>(),
        size_of::<Prepared>()
    );
    let mut baseline = None;
    for unrelated in [1, 8, 32] {
        let mut source = String::from("\nfn selected<T>(x:T)->T{x}");
        for index in 0..unrelated {
            writeln!(source, "\nfn unrelated{index}<T>(x:T)->T{{x}}")?;
        }
        let mut work = ReservationWork::default();
        let (session, owner) = start(&source, "selected", &mut work)?;
        let mut session = ready(session.defining_environment(owner, &mut work)?)?;
        let before = work.steps;
        let bytes = work.retained;
        let extent = session.facts.types.interned_extent();
        let counters = session
            .recipes
            .as_ref()
            .ok_or("actual cursor")?
            .cursor
            .counters();
        for _ in 0..1000 {
            session = ready(session.defining_environment(owner, &mut work)?)?;
        }
        let cost = work.steps - before;
        if let Some(previous) = baseline {
            assert_eq!(cost, previous);
        } else {
            baseline = Some(cost);
        }
        assert_eq!(work.retained, bytes);
        assert_eq!(session.facts.types.interned_extent(), extent);
        assert_eq!(
            session
                .recipes
                .as_ref()
                .ok_or("actual cursor")?
                .cursor
                .counters(),
            counters
        );
        eprintln!(
            "private defining unrelated={unrelated} warm1000={cost} bytes={}",
            environments(&session)?.retained_bytes()
        );
    }
    for width in [1, 8, 32] {
        let mut source = String::from("\nfn wide<");
        for index in 0..width {
            if index != 0 {
                source.push(',');
            }
            write!(source, "T{index}")?;
        }
        source.push_str(">(x:u8)->u8{x}");
        let mut work = ReservationWork::default();
        let (session, owner) = start(&source, "wide", &mut work)?;
        let session = ready(session.defining_environment(owner, &mut work)?)?;
        let env = environments(&session)?;
        let scope = env
            .scopes
            .iter()
            .flatten()
            .next()
            .ok_or("actual complete scope")?;
        assert_eq!(scope.parameters.len(), width);
        assert!(env.applications.iter().flatten().count() >= width);
        eprintln!(
            "private defining width={width} bytes={} steps={} peak={}",
            env.retained_bytes(),
            work.steps,
            work.peak
        );
    }
    Ok(())
}
#[test]
fn private_trait_input_costs_cover_real_lazy_growth_and_capacity_hits() -> Result {
    let (report, _, _) = actual(true)?;
    let definition = report
        .resolution
        .definitions
        .iter()
        .enumerate()
        .find(|(_, definition)| {
            definition.kind == crate::resolve::model::DefKind::Trait
                && report.package.names.get(definition.name) == "Sized"
        })
        .map(|(index, _)| u32::try_from(index).map(crate::types::ItemId))
        .transpose()?
        .ok_or("actual declared Sized identity")?;
    // This exercises storage costs only; no resulting application is supplied
    // to an Environment, implementation solver or executable permission query.
    let mut store = crate::types::Store::default();
    for width in [1, 8, 32] {
        let values =
            vec![
                Argument::Type(crate::types::TypeId::primitive(crate::types::Primitive::U8));
                width
            ];
        let mut work = ReservationWork::default();
        let before = store.trait_input_bytes();
        let arguments_growth = store.trait_arguments_growth(values.len());
        work.charge(values.len().saturating_add(arguments_growth.work))?;
        work.reserve(arguments_growth.bytes)?;
        let arguments = store.intern_arguments(&values)?;
        assert!(store.trait_input_bytes() - before <= arguments_growth.bytes);
        let app = crate::types::traits::Application {
            item: definition,
            arguments,
        };
        let before = store.trait_input_bytes();
        let growth = store.trait_application_growth();
        work.charge(width.saturating_add(growth.work))?;
        work.reserve(growth.bytes)?;
        let actual = store.intern_trait_application(app)?;
        assert!(store.trait_input_bytes() - before <= growth.bytes);
        let bytes = store.trait_input_bytes();
        for _ in 0..1000 {
            assert_eq!(store.find_mixed_arguments(&values), Some(arguments));
            assert_eq!(store.find_trait_application(app), Some(actual));
        }
        assert_eq!(store.trait_input_bytes(), bytes);
    }
    Ok(())
}
