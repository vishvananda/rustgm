//! Real nonzero Declaration bodies stay separate from regional/default proof.
use super::super::{Demand, NumberedBody, Session, inherited_tests as inherited};
use super::*;
use crate::unpaid_cases::Result;
use crate::{resolve::crates::ReservationWork, support::crates::owner_catalog::DeclarationRef};
use std::fmt::Write as _;

fn named(session: &Session, name: &str) -> Result<DeclarationRef> {
    let rows = session.inventory.artifacts()[0]
        .source_descriptor()
        .ok_or("actual rows")?
        .rows();
    rows.functions
        .iter()
        .find(|header| {
            rows.declarations[header.declaration.0 as usize]
                .name
                .as_deref()
                == Some(name)
        })
        .map(|header| header.declaration)
        .ok_or_else(|| "actual named original".into())
}
fn entry(session: &Session, owner: DeclarationRef) -> Result<(&Ready, usize, &NumberedBody)> {
    let body = inherited::environment(session)?
        .bodies
        .iter()
        .flatten()
        .find(|body| match body.as_ref() {
            Body::Numbered { expected, .. } => expected.header.declaration == owner,
            _ => false,
        })
        .ok_or("actual numbered body")?;
    let Body::Numbered {
        expected,
        scope,
        product,
    } = body.as_ref()
    else {
        return Err("actual numbered variant".into());
    };
    Ok((expected, *scope, product))
}
pub(in super::super) fn controls(
    session: Box<Session>,
    owner: DeclarationRef,
    work: &mut ReservationWork,
) -> Result {
    positive(&session, owner)?;
    warm(session, owner, work)?;
    opponents()?;
    blocked()?;
    scaling()?;
    failed_prepayment()?;
    universal_widths()?;
    Ok(())
}
fn positive(session: &Session, owner: DeclarationRef) -> Result {
    let (expected, scope, product) = entry(session, owner)?;
    let rows = session.inventory.artifacts()[0]
        .source_descriptor()
        .ok_or("actual rows")?
        .rows();
    let s::FunctionBody::Original(body) = expected.header.body else {
        return Err("actual body".into());
    };
    let input = &rows.body_inputs[body.0 as usize];
    assert_eq!(input.environment, s::EnvironmentOrigin::Declaration(owner));
    assert!(input.assumptions.is_empty() && input.type_assumptions.is_empty());
    assert!(expected.header.regions.total > 0);
    let declaration = inherited::environment(session)?.scopes[scope]
        .get()
        .map(Box::as_ref)
        .ok_or("actual scope")?;
    assert!(
        declaration
            .environment
            .neutral_alias_finish_bound(0)
            .is_ok()
    );
    assert!(Arc::ptr_eq(&declaration.environment, &product.declaration));
    let unit = &rows.body_units[body.0 as usize];
    if unit.aliases.as_ref().is_none_or(Vec::is_empty) {
        assert!(Arc::ptr_eq(&product.environment, &declaration.environment));
    } else {
        assert!(!Arc::ptr_eq(&product.environment, &declaration.environment));
    }
    let source = session
        .facts
        .generics
        .body_environments
        .values()
        .find(|environment| environment.body_aliases().is_some())
        .ok_or("actual original-family source ledger")?;
    assert!(source.neutral_alias_finish_bound(0).is_err());
    crate::types::parameters::environment::alias_tests::check(
        &declaration.environment,
        source
            .body_aliases()
            .ok_or("actual source ledger")?
            .ordered(),
    )?;
    assert!(
        session.facts.definitions[session.old_definitions..]
            .iter()
            .all(Option::is_none)
    );
    eprintln!(
        "neutral defining layouts key={} product={} body={} owner={} prepared={} state={} session={} universals={} aliases={:?} owner_bytes={}",
        size_of::<numbering_records::Key>(),
        size_of::<NumberedBody>(),
        size_of::<Body>(),
        size_of::<Owner>(),
        size_of::<Prepared>(),
        size_of::<super::super::State>(),
        size_of::<Session>(),
        unit.universals,
        unit.aliases.as_ref().map(Vec::len),
        inherited::environment(session)?.retained_bytes()
    );
    Ok(())
}
fn opponents() -> Result {
    let source = "trait Holder<V>{fn carry<U>(value:U)->U{value}fn borrow(value:&V)->&V{value}fn twin(value:&V)->&V{value}} impl Holder<u8> for u8{}";
    for opponent in 0..6 {
        let mut work = ReservationWork::default();
        let inherited::Case {
            session,
            contracts: _contracts,
            ..
        } = inherited::start(source, &mut work)?;
        let owner = named(&session, "borrow")?;
        let mut session = inherited::ready(session.defining_environment(owner, &mut work)?)?;
        match opponent {
            0 => session.facts.types = crate::types::Store::default(),
            1 => session.inventory = crate::unpaid_cases::actual(true)?.2,
            2 => {
                let environments = session
                    .recipes
                    .as_mut()
                    .and_then(|state| state.callables.as_mut())
                    .and_then(|prepared| prepared.environments.as_mut())
                    .ok_or("actual owner")?;
                let Body::Numbered { product, .. } = environments
                    .bodies
                    .iter_mut()
                    .flatten()
                    .find(|body| matches!(body.as_ref(), Body::Numbered { .. }))
                    .ok_or("actual product")?
                    .as_mut()
                else {
                    return Err("actual numbered product".into());
                };
                product.environment = Arc::new(product.environment.as_ref().clone());
            }
            3 => super::super::super::numbering_records::tests::remove_selected_record(
                &mut session,
                owner,
                &mut work,
            )?,
            4 => session = wrong_body(session, owner, &mut work)?,
            _ => {
                let table = session
                    .recipes
                    .as_mut()
                    .and_then(|state| state.callables.as_mut())
                    .ok_or("actual prepared scopes")?;
                std::rc::Rc::get_mut(&mut table.scopes).ok_or("exclusive actual scope index")?
                    [owner.0 as usize] = None;
            }
        }
        let before = work.steps;
        assert!(session.defining_environment(owner, &mut work).is_err());
        assert!(work.steps > before);
    }
    Ok(())
}
fn wrong_body(
    mut session: Box<Session>,
    owner: DeclarationRef,
    work: &mut ReservationWork,
) -> Result<Box<Session>> {
    let foreign = named(&session, "twin")?;
    session = inherited::ready(session.defining_environment(foreign, work)?)?;
    let environments = session
        .recipes
        .as_mut()
        .and_then(|state| state.callables.as_mut())
        .and_then(|prepared| prepared.environments.as_mut())
        .ok_or("actual owner")?;
    let position = |owner| {
        environments.bodies.iter().position(|body|
        matches!(body.as_deref(), Some(Body::Numbered { expected, .. }) if expected.header.declaration == owner))
    };
    let actual = position(owner).ok_or("actual numbered row")?;
    let other = position(foreign).ok_or("actual other numbered row")?;
    let (left, right) = environments.bodies.split_at_mut(actual.max(other));
    let a = left[actual.min(other)]
        .as_deref_mut()
        .ok_or("actual first body")?;
    let b = right[0].as_deref_mut().ok_or("actual second body")?;
    let (
        Body::Numbered {
            expected: a_expected,
            product: a_product,
            ..
        },
        Body::Numbered {
            expected: b_expected,
            product: b_product,
            ..
        },
    ) = (a, b)
    else {
        return Err("actual numbered variants".into());
    };
    assert_eq!(a_expected.pair.original, b_expected.pair.original);
    std::mem::swap(a_product, b_product);
    Ok(session)
}
fn blocked() -> Result {
    let suffix = format!(
        "{} trait Regional{{fn regional<U:'static>(value:&U)->&U{{value}}}} fn plain(value:&u8)->&u8{{value}}",
        inherited::HOLDER
    );
    let mut work = ReservationWork::default();
    let inherited::Case {
        mut session,
        contracts: _contracts,
        ..
    } = inherited::start(&suffix, &mut work)?;
    for name in ["regional", "plain", "clone_from"] {
        let owner = named(&session, name)?;
        let (next, status) = session.defining_environment(owner, &mut work)?;
        session = next;
        assert!(matches!(status, Demand::Blocked(_)), "{name}: {status:?}");
    }
    Ok(())
}
fn scaling() -> Result {
    let mut common = None;
    for unrelated in [1, 8, 32] {
        let mut source = inherited::HOLDER.to_owned();
        for index in 0..unrelated {
            writeln!(source, "fn other{index}(value:u8)->u8{{value}}")?;
        }
        let mut work = ReservationWork::default();
        let inherited::Case {
            session,
            contracts: _contracts,
            ..
        } = inherited::start(&source, &mut work)?;
        let owner = named(&session, "borrow")?;
        let session = inherited::ready(session.defining_environment(owner, &mut work)?)?;
        let (session, cost) = warm(session, owner, &mut work)?;
        if let Some(old) = common {
            assert_eq!(cost, old);
        } else {
            common = Some(cost);
        }
        eprintln!(
            "neutral unrelated={unrelated} warm1000={cost} owner_bytes={} peak={}",
            inherited::environment(&session)?.retained_bytes(),
            work.peak
        );
    }
    for width in [1, 8, 32] {
        let mut source = String::from("trait Holder<");
        for index in 0..width {
            if index != 0 {
                source.push(',');
            }
            write!(source, "V{index}")?;
        }
        source.push_str(">{fn carry<U>(value:U)->U{value}fn borrow(value:&V0)->&V0{value}}");
        let mut work = ReservationWork::default();
        let inherited::Case {
            session,
            contracts: _contracts,
            ..
        } = inherited::start(&source, &mut work)?;
        let owner = named(&session, "borrow")?;
        let session = inherited::ready(session.defining_environment(owner, &mut work)?)?;
        let (expected, ..) = entry(&session, owner)?;
        assert_eq!(expected.formals, width + 1);
        let (session, cost) = warm(session, owner, &mut work)?;
        eprintln!(
            "neutral width={width} warm1000={cost} owner_bytes={} peak={}",
            inherited::environment(&session)?.retained_bytes(),
            work.peak
        );
    }
    Ok(())
}
fn warm(
    mut session: Box<Session>,
    owner: DeclarationRef,
    work: &mut ReservationWork,
) -> Result<(Box<Session>, usize)> {
    let before = work.steps;
    let retained = work.retained;
    let extent = session.facts.types.interned_extent();
    let counters = session
        .recipes
        .as_ref()
        .ok_or("actual cursor")?
        .cursor
        .counters();
    for _ in 0..1000 {
        session = inherited::ready(session.defining_environment(owner, work)?)?;
    }
    assert_eq!(work.retained, retained);
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
fn failed_prepayment() -> Result {
    let mut work = ReservationWork::default();
    let inherited::Case {
        session,
        contracts: _contracts,
        ..
    } = inherited::start(inherited::HOLDER, &mut work)?;
    let owner = named(&session, "borrow")?;
    let session = inherited::ready(session.record_numbering(owner, &mut work)?)?;
    work.retained = crate::support::crates::MAX_BYTES;
    let before = work.steps;
    assert!(session.defining_environment(owner, &mut work).is_err());
    assert!(work.steps > before && work.peak > crate::support::crates::MAX_BYTES);
    Ok(())
}

fn universal_widths() -> Result {
    let mut common = None;
    for width in [1, 8, 32] {
        let mut source = String::from("trait Holder<V>{fn carry<U>(value:U)->U{value}fn borrow(");
        for index in 0..width {
            if index != 0 {
                source.push(',');
            }
            write!(source, "value{index}:&V")?;
        }
        source.push_str("){}}");
        let mut work = ReservationWork::default();
        let inherited::Case {
            session,
            contracts: _contracts,
            ..
        } = inherited::start(&source, &mut work)?;
        let owner = named(&session, "borrow")?;
        let session = inherited::ready(session.defining_environment(owner, &mut work)?)?;
        assert_eq!(
            entry(&session, owner)?.0.header.regions.total as usize,
            width
        );
        let (session, cost) = warm(session, owner, &mut work)?;
        if let Some(previous) = common {
            assert_eq!(cost, previous);
        } else {
            common = Some(cost);
        }
        eprintln!(
            "neutral universals={width} warm1000={cost} owner_bytes={} peak={}",
            inherited::environment(&session)?.retained_bytes(),
            work.peak
        );
    }
    Ok(())
}
