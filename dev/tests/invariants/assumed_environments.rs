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
    plain_contract()?;
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
    session = ready(session.defining_environment(plain, &mut work)?)?;
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
fn plain_contract() -> Result {
    let mut source_owner = None;
    let (report, facts, inventory) =
        crate::unpaid_cases::extended_with("\nfn plain(x:u8)->u8{x}", |report, facts, _, _| {
            let owner = report
                .resolution
                .definitions
                .iter()
                .enumerate()
                .find(|(_, definition)| {
                    definition.kind == crate::resolve::model::DefKind::Function
                        && report.package.names.get(definition.name) == "plain"
                })
                .map(|(index, _)| u32::try_from(index).map(crate::resolve::model::DefId))
                .transpose()?
                .ok_or("actual source plain owner")?;
            assert!(!facts.generics.environments.contains_key(&owner));
            source_owner = Some(owner);
            Ok(())
        })?;
    let owner = named(&inventory, "plain")?;
    let rows = inventory.artifacts()[0]
        .source_descriptor()
        .ok_or("actual rows")?
        .rows();
    let header = rows
        .functions
        .iter()
        .find(|row| row.declaration == owner)
        .ok_or("plain header")?;
    let s::FunctionBody::Original(body) = header.body else {
        return Err("actual plain original".into());
    };
    let original = rows.originals[body.0 as usize].clone();
    let input = rows.body_inputs[body.0 as usize].clone();
    let unit = rows.body_units[body.0 as usize].clone();
    assert_eq!(input.environment, s::EnvironmentOrigin::Contract);
    assert_eq!(
        input
            .builtins
            .as_ref()
            .ok_or("actual raw builtin debt")?
            .environment,
        s::EnvironmentOrigin::Contract
    );
    let maps = facts.generics.environments.len();
    let mut work = ReservationWork::default();
    let session = Session::prepare(report, facts, inventory, 0, &mut work)?.boxed(&mut work)?;
    let session = ready(session.defining_environment(owner, &mut work)?)?;
    assert_eq!(session.facts.generics.environments.len(), maps);
    assert!(
        !session
            .facts
            .generics
            .environments
            .contains_key(&source_owner.ok_or("actual source owner")?)
    );
    let environments = environments(&session)?;
    let Body::Contract {
        expected,
        scope,
        product,
        ..
    } = environments
        .bodies
        .iter()
        .flatten()
        .next()
        .ok_or("actual Contract record")?
        .as_ref()
    else {
        return Err("Contract must retain its own body product".into());
    };
    assert_eq!(expected.header.declaration, owner);
    let scope = environments.scopes[*scope]
        .as_deref()
        .ok_or("actual scope")?;
    scope.require_empty(*expected)?;
    assert!(product.environment.is_neutral());
    assert!(!Arc::ptr_eq(&product.environment, &scope.environment));
    assert!(Arc::ptr_eq(&product.declaration, &scope.environment));
    let rows = session.inventory.artifacts()[0]
        .source_descriptor()
        .ok_or("retained rows")?
        .rows();
    assert_eq!(rows.originals[body.0 as usize], original);
    assert_eq!(rows.body_inputs[body.0 as usize], input);
    assert_eq!(rows.body_units[body.0 as usize], unit);
    session.require(&mut work)?;
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
            let scope = match record.as_mut() {
                Body::Ready { scope, .. } | Body::Contract { scope, .. } => scope,
                Body::Blocked { .. } => return Err("actual ready record".into()),
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
                    }
                    | Body::Contract {
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
    contract_opponents()?;
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
fn contract_opponents() -> Result {
    for opponent in 0..10 {
        let mut work = ReservationWork::default();
        let (session, owner) = start("\nfn plain(x:u8)->u8{x}", "plain", &mut work)?;
        let mut session = ready(session.defining_environment(owner, &mut work)?)?;
        if opponent < 6 {
            session = change_owner(session, owner, opponent, &mut work)?;
        } else {
            let foreign = named(&session.inventory, "identity")?;
            let rows = session.inventory.artifacts()[0]
                .source_descriptor()
                .ok_or("actual full family")?
                .rows();
            let foreign_body = rows
                .functions
                .iter()
                .find(|header| header.declaration == foreign)
                .ok_or("actual other original")?
                .body;
            let owner = session
                .recipes
                .as_mut()
                .and_then(|state| state.callables.as_mut())
                .and_then(|prepared| prepared.environments.as_mut())
                .ok_or("actual Contract owner")?;
            let record = owner
                .bodies
                .iter_mut()
                .flatten()
                .next()
                .ok_or("actual Contract body")?;
            let Body::Contract {
                product,
                scope,
                expected,
            } = record.as_mut()
            else {
                return Err("actual Contract memo".into());
            };
            match opponent {
                6 => product.seal = originals::Origin::Declaration,
                7 => product.environment = Arc::clone(&product.declaration),
                8 => {
                    owner.scopes[*scope] = None;
                }
                _ => expected.header.body = foreign_body,
            }
        }
        let before = work.steps;
        assert!(session.defining_environment(owner, &mut work).is_err());
        assert!(work.steps > before);
    }
    let mut work = ReservationWork::default();
    let (session, owner) = start("\nfn plain(x:u8)->u8{x}", "plain", &mut work)?;
    let session = ready(session.defining_environment(owner, &mut work)?)?;
    let Body::Contract { product, .. } = environments(&session)?
        .bodies
        .iter()
        .flatten()
        .next()
        .ok_or("actual paid Contract")?
        .as_ref()
    else {
        return Err("actual sealed empty Contract".into());
    };
    let originals::Origin::Contract(seal) = product.seal else {
        return Err("actual Contract seal".into());
    };
    work.retained = crate::support::crates::MAX_BYTES;
    let mut accounting = Observed {
        work: &mut work,
        allocation: Allocation::default(),
    };
    assert!(crate::sema::crates::typed_headers::staged_headers::rebuilt_environments::ContractParts::private_empty(seal)
        .build(&session.facts, &mut accounting).is_err());
    assert_eq!(accounting.allocation.bytes, 0);
    assert!(work.peak > crate::support::crates::MAX_BYTES);
    let mut work = ReservationWork::default();
    let (session, owner) = start("\nfn plain(x:u8)->u8{x}", "plain", &mut work)?;
    let session = ready(session.callable_template(owner, &mut work)?)?;
    work.retained = crate::support::crates::MAX_BYTES;
    assert!(session.defining_environment(owner, &mut work).is_err());
    assert!(work.peak > crate::support::crates::MAX_BYTES);
    Ok(())
}

#[test]
fn private_defining_environments_bound_selected_and_warm_work() -> Result {
    eprintln!(
        "private defining layouts: owner={} scope={} body={} contract_product={} templates={} state={} session={} body_slot={} scope_slot={} ready={}",
        size_of::<Owner>(),
        size_of::<Scope>(),
        size_of::<Body>(),
        size_of::<ContractBody>(),
        size_of::<Prepared>(),
        size_of::<State>(),
        size_of::<Session>(),
        size_of::<Option<Box<Body>>>(),
        size_of::<Option<Box<Scope>>>(),
        size_of::<Ready>()
    );
    contract_growth()?;
    let mut baseline = None;
    for unrelated in [1, 8, 32] {
        let mut source = String::from("\nfn selected<T>(x:T)->T{x}");
        for index in 0..unrelated {
            writeln!(source, "\nfn unrelated{index}<T>(x:T)->T{{x}}")?;
        }
        let mut work = ReservationWork::default();
        let (session, owner) = start(&source, "selected", &mut work)?;
        let mut session = ready(session.defining_environment(owner, &mut work)?)?;
        let (next, cost) = warm_environment(session, owner, &mut work)?;
        session = next;
        if let Some(previous) = baseline {
            assert_eq!(cost, previous);
        } else {
            baseline = Some(cost);
        }
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
fn warm_environment(
    mut session: Box<Session>,
    owner: DeclarationRef,
    work: &mut ReservationWork,
) -> Result<(Box<Session>, usize)> {
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
        session = ready(session.defining_environment(owner, work)?)?;
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
    Ok((session, work.steps - before))
}

fn contract_growth() -> Result {
    let mut baseline = None;
    for unrelated in [1, 8, 32] {
        let mut source = String::from("\nfn selected(x:u8)->u8{x}");
        for index in 0..unrelated {
            writeln!(source, "\nfn other{index}(x:u8)->u8{{x}}")?;
        }
        let mut work = ReservationWork::default();
        let (session, owner) = start(&source, "selected", &mut work)?;
        let mut session = ready(session.defining_environment(owner, &mut work)?)?;
        let (next, cost) = warm_environment(session, owner, &mut work)?;
        session = next;
        if let Some(previous) = baseline {
            assert_eq!(cost, previous);
        } else {
            baseline = Some(cost);
        }
        let environments = environments(&session)?;
        let Body::Contract { product, .. } = environments
            .bodies
            .iter()
            .flatten()
            .next()
            .ok_or("actual selected Contract")?
            .as_ref()
        else {
            return Err("actual Contract product".into());
        };
        let body_allocation = product
            .environment
            .retain_owned_from(
                &product.declaration,
                &mut crate::types::parameters::environment::Retained::default(),
            )
            .bytes
            + 2 * size_of::<usize>();
        eprintln!(
            "private Contract actual_environment_payload={body_allocation} cold_product={}",
            size_of::<ContractBody>()
        );
        eprintln!(
            "private Contract unrelated={unrelated} warm1000={cost} bytes={} peak={} bodies_cap={} body_dense_bytes={} allocated_bodies={} scopes_cap={} scope_dense_bytes={} applications_cap={} applications_bytes={}",
            environments.retained_bytes(),
            work.peak,
            environments.bodies.capacity(),
            retained::vector(&environments.bodies),
            environments.bodies.iter().flatten().count(),
            environments.scopes.capacity(),
            retained::vector(&environments.scopes),
            environments.applications.capacity(),
            retained::vector(&environments.applications)
        );
    }
    Ok(())
}

#[test]
fn private_trait_input_costs_cover_real_lazy_growth_and_capacity_hits() -> Result {
    vector_boundary::<u8>()?;
    vector_boundary::<u16>()?;
    vector_boundary::<u32>()?;
    vector_boundary::<u64>()?;
    vector_boundary::<()>()?;
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

fn vector_boundary<T: Default>() -> Result {
    let mut values = Vec::<T>::new();
    let mut work = ReservationWork::default();
    for _ in 0..33 {
        let before = crate::support::retained::vector(&values);
        let growth =
            crate::support::retained::Growth::vector::<T>(values.len(), values.capacity(), 1);
        work.charge(growth.work.saturating_add(1))?;
        work.reserve(growth.bytes)?;
        values.push(T::default());
        let actual = crate::support::retained::vector(&values).saturating_sub(before);
        assert!(actual <= growth.bytes);
        if size_of::<T>() == 0 {
            assert_eq!((actual, growth.bytes, growth.work), (0, 0, 0));
        }
    }
    Ok(())
}
