//! Inherited Self/V/U come from the genuine numbered ordinary-trait producer.
use super::*;
use crate::unpaid_cases::{Result, TraitFixture, trait_extended};
use std::fmt::Write as _;
const HOLDER: &str = "\npub trait Holder<V>{fn carry<U>(value:U)->U{value}fn borrow(value:&V)->&V{value}}\nimpl Holder<u8> for u8{}";

struct Case {
    session: Box<Session>,
    owner: DeclarationRef,
    contracts: s::HeaderAttributeColumns,
}
fn start(suffix: &str, work: &mut ReservationWork) -> Result<Case> {
    let TraitFixture {
        report,
        facts,
        inventory,
        contracts,
        contract_bytes,
        work: source_work,
    } = trait_extended(suffix)?;
    let rows = inventory.artifacts()[0]
        .source_descriptor()
        .ok_or("actual provided input")?
        .rows();
    assert!(
        rows.header_attributes().is_none(),
        "raw Provided rows cannot imply installed attribute knowledge"
    );
    let owner = rows
        .functions
        .iter()
        .find(|header| {
            rows.declarations[header.declaration.0 as usize]
                .name
                .as_deref()
                == Some("carry")
        })
        .ok_or("actual provided method")?
        .declaration;
    work.reserve(contract_bytes)?;
    eprintln!(
        "inherited source steps={} reserved={} scratch={} retained_columns={contract_bytes}",
        source_work.steps, source_work.reserved_bytes, source_work.scratch_bytes
    );
    let session = Session::prepare(report, facts, inventory, 0, work)?.boxed(work)?;
    Ok(Case {
        session,
        owner,
        contracts,
    })
}
fn ready(progress: Progress) -> Result<Box<Session>> {
    let (session, status) = progress;
    if status != Demand::Ready(()) {
        return Err("actual inherited declaration environment blocked".into());
    }
    Ok(session)
}
fn environment(session: &Session) -> Result<&Owner> {
    session
        .recipes
        .as_ref()
        .and_then(|state| state.callables.as_ref())
        .and_then(|prepared| prepared.environments.as_deref())
        .ok_or_else(|| "actual inherited environment".into())
}
#[test]
fn inherited_trait_inputs_prepare_actual_self_formals_and_bound_warm_work() -> Result {
    let mut work = ReservationWork::default();
    let Case {
        session,
        owner,
        contracts,
    } = start(HOLDER, &mut work)?;
    let mut session = ready(session.defining_environment(owner, &mut work)?)?;
    check_scope(&session, owner, &contracts, &mut work)?;
    let rows = session.inventory.artifacts()[0]
        .source_descriptor()
        .ok_or("actual rows")?
        .rows();
    let borrowed = rows
        .functions
        .iter()
        .find(|header| {
            rows.declarations[header.declaration.0 as usize]
                .name
                .as_deref()
                == Some("borrow")
        })
        .ok_or("actual nonzero provided method")?
        .declaration;
    let (next, status) = session.defining_environment(borrowed, &mut work)?;
    session = next;
    assert!(matches!(
        status,
        Demand::Blocked(Dependency::OwnedBodyDomain(_))
    ));
    assert_eq!(environment(&session)?.bodies.iter().flatten().count(), 1);
    inherited_opponents()?;
    growth()?;
    Ok(())
}
fn check_scope(
    session: &Session,
    owner: DeclarationRef,
    contracts: &s::HeaderAttributeColumns,
    work: &mut ReservationWork,
) -> Result {
    let rows = session.inventory.artifacts()[0]
        .source_descriptor()
        .ok_or("actual rows")?
        .rows();
    let holder = rows
        .traits
        .iter()
        .find(|header| {
            rows.declarations[header.declaration.0 as usize]
                .name
                .as_deref()
                == Some("Holder")
        })
        .ok_or("actual Holder")?;
    assert!(
        !rows
            .roles
            .iter()
            .any(|role| role.declaration.declaration == holder.declaration)
    );
    let environments = environment(session)?;
    let Body::Ready {
        expected,
        scope,
        environment,
    } = environments
        .bodies
        .iter()
        .flatten()
        .next()
        .ok_or("actual member body")?
        .as_ref()
    else {
        return Err("actual Declaration body origin".into());
    };
    assert_eq!(expected.header.declaration, owner);
    let scope = environments.scopes[*scope]
        .as_deref()
        .ok_or("actual member scope")?;
    assert!(Arc::ptr_eq(environment, &scope.environment));
    assert_eq!(scope.parameters.len(), 3);
    let view = session.inputs.recipe_view(&session.inventory, 0, work)?;
    let s::FunctionBody::Original(body) = expected.header.body else {
        return Err("actual body".into());
    };
    for (&slot, (parameter, _)) in rows.body_inputs[body.0 as usize]
        .parameters
        .iter()
        .zip(scope.parameters.iter().rev())
    {
        let raw = rows.slots[slot.0 as usize];
        let actual = session
            .facts
            .types
            .parameter(*parameter)
            .ok_or("actual rigid formal")?;
        assert_eq!(
            actual.owner,
            crate::types::ItemId(view.definition(raw.owner, work)?.0)
        );
        assert_eq!(actual.index, raw.index);
    }
    let holder_item = crate::types::ItemId(view.definition(holder.declaration, work)?.0);
    assert!(scope.clauses.iter().any(|clause| matches!(clause, Clause::Predicate(predicate)
        if matches!(predicate.bound, crate::types::parameters::predicates::Bound::Trait(application)
            if session.facts.types.trait_application(application).is_some_and(|application| application.item == holder_item)))));
    let inherited = environments
        .scopes
        .iter()
        .flatten()
        .find(|scope| scope.raw.owner == holder.scope)
        .ok_or("actual trait scope")?;
    assert!(inherited.inputs.iter().all(|input| matches!(
        input.source,
        crate::resolve::body_origin::ProofSite::Imported(_)
    )));
    let index = rows
        .functions
        .iter()
        .position(|header| header.declaration == owner)
        .ok_or("actual row")?;
    assert_eq!(contracts.functions[index].declaration, owner);
    assert!(contracts.functions[index].must_use.is_none());
    assert!(
        session.facts.definitions[session.old_definitions..]
            .iter()
            .all(Option::is_none)
    );
    session.require(work)?;
    Ok(())
}
fn inherited_opponents() -> Result {
    for opponent in 0..6 {
        let mut work = ReservationWork::default();
        let Case {
            session,
            owner,
            contracts: _contracts,
        } = start(HOLDER, &mut work)?;
        if opponent == 4 {
            missing_parent(session, owner, &mut work)?;
            continue;
        }
        let mut session = ready(session.defining_environment(owner, &mut work)?)?;
        match opponent {
            0 => session.facts.types = crate::types::Store::default(),
            1 => {
                let (_, _, foreign) = crate::unpaid_cases::actual(true)?;
                session.inventory = foreign;
            }
            2 => {
                session
                    .recipes
                    .as_mut()
                    .and_then(|state| state.callables.as_mut())
                    .ok_or("actual template")?
                    .scopes[owner.0 as usize] = None;
            }
            3 => work.retained = crate::support::crates::MAX_BYTES,
            _ => {
                let environments = session
                    .recipes
                    .as_mut()
                    .and_then(|state| state.callables.as_mut())
                    .and_then(|prepared| prepared.environments.as_mut())
                    .ok_or("actual inherited owner")?;
                let Body::Ready { scope, .. } = environments
                    .bodies
                    .iter()
                    .flatten()
                    .next()
                    .ok_or("actual body")?
                    .as_ref()
                else {
                    return Err("actual inherited Declaration".into());
                };
                environments.scopes[*scope]
                    .as_mut()
                    .ok_or("actual scope")?
                    .parameters
                    .pop()
                    .ok_or("actual complete formal")?;
            }
        }
        let before = work.steps;
        if opponent == 3 {
            // Warm lookup is allocation-free; force a genuine cold second request.
            let rows = session.inventory.artifacts()[0]
                .source_descriptor()
                .ok_or("actual rows")?
                .rows();
            let plain = rows
                .functions
                .iter()
                .find(|header| {
                    rows.declarations[header.declaration.0 as usize]
                        .name
                        .as_deref()
                        == Some("identity")
                })
                .ok_or("actual other original")?
                .declaration;
            assert!(session.defining_environment(plain, &mut work).is_err());
            assert!(work.peak > crate::support::crates::MAX_BYTES);
        } else {
            assert!(session.defining_environment(owner, &mut work).is_err());
        }
        assert!(work.steps > before);
    }
    Ok(())
}
fn missing_parent(
    session: Box<Session>,
    owner: DeclarationRef,
    work: &mut ReservationWork,
) -> Result {
    let mut session = ready(session.signature_pair(owner, work)?)?;
    let view = session.inputs.recipe_view(&session.inventory, 0, work)?;
    let rows = view.rows()?;
    let header = rows
        .functions
        .iter()
        .find(|header| header.declaration == owner)
        .ok_or("actual member")?;
    let parent = header.parent.ok_or("actual inherited trait owner")?;
    let mut table = Prepared::new(&view, work)?;
    table.scopes[parent.0 as usize] = None;
    session
        .recipes
        .as_mut()
        .ok_or("actual persistent cursor")?
        .callables = Some(Box::new(table));
    let before = work.steps;
    assert!(session.callable_template(owner, work).is_err());
    assert!(work.steps > before);
    Ok(())
}

fn growth() -> Result {
    let mut baseline = None;
    for unrelated in [1, 8, 32] {
        let mut source = HOLDER.to_owned();
        for index in 0..unrelated {
            writeln!(source, "\nfn other{index}(value:u8)->u8{{value}}")?;
        }
        let mut work = ReservationWork::default();
        let Case {
            session,
            owner,
            contracts: _contracts,
        } = start(&source, &mut work)?;
        let session = ready(session.defining_environment(owner, &mut work)?)?;
        let (session, steps) = super::tests::warm_environment(session, owner, &mut work)?;
        if let Some(previous) = baseline {
            assert_eq!(steps, previous);
        } else {
            baseline = Some(steps);
        }
        eprintln!(
            "inherited unrelated={unrelated} warm1000={steps} owner_bytes={} peak={}",
            environment(&session)?.retained_bytes(),
            work.peak
        );
    }
    for width in [1, 8, 32] {
        let mut source = String::from("\npub trait Holder<");
        for index in 0..width {
            if index != 0 {
                source.push(',');
            }
            write!(source, "V{index}")?;
        }
        source.push_str(">{fn carry<U>(value:U)->U{value}}");
        let mut work = ReservationWork::default();
        let Case {
            session,
            owner,
            contracts: _contracts,
        } = start(&source, &mut work)?;
        let session = ready(session.defining_environment(owner, &mut work)?)?;
        let (session, steps) = super::tests::warm_environment(session, owner, &mut work)?;
        let environments = environment(&session)?;
        let count = environments
            .scopes
            .iter()
            .flatten()
            .map(|scope| scope.parameters.len())
            .max()
            .ok_or("actual full inherited scope")?;
        assert_eq!(count, width + 2);
        eprintln!(
            "inherited width={width} warm1000={steps} owner_bytes={} peak={}",
            environments.retained_bytes(),
            work.peak
        );
    }
    Ok(())
}
