//! Scoped inspection clones only selected bodies after their actual region/loan proof.
use crate::{
    body::ir::{Body, Owner},
    resolve::{model::DefId, package::Error},
};
use std::cell::RefCell;
type TestResult<T> = Result<T, Box<dyn std::error::Error>>;
struct Capture {
    targets: Vec<DefId>,
    bodies: Vec<Body>,
    instances: bool,
    count: usize,
}
thread_local! {
    static CAPTURE: RefCell<Option<Capture>> = const { RefCell::new(None) };
}
struct Scope;
impl Drop for Scope {
    fn drop(&mut self) {
        CAPTURE.with(|slot| {
            drop(slot.replace(None));
        });
    }
}
pub(crate) fn during<T>(
    targets: &[DefId],
    prepare: impl FnOnce() -> Result<T, Error>,
) -> TestResult<(T, Vec<Body>)> {
    if targets.len() > 2 || (targets.len() == 2 && targets[0] == targets[1]) {
        return Err("materialized observer requires at most two distinct actual targets".into());
    }
    observe(targets, targets.len(), false, prepare)
}
/// A bounded source scaling observer retains distinct completed full instances.
pub(crate) fn instances<T>(
    targets: &[DefId],
    count: usize,
    prepare: impl FnOnce() -> Result<T, Error>,
) -> TestResult<(T, Vec<Body>)> {
    if targets.is_empty() || targets.len() > 6 || count < targets.len() || count > 15 {
        return Err("materialized instance observation exceeds its declared source bound".into());
    }
    observe(targets, count, true, prepare)
}
fn observe<T>(
    targets: &[DefId],
    count: usize,
    instances: bool,
    prepare: impl FnOnce() -> Result<T, Error>,
) -> TestResult<(T, Vec<Body>)> {
    CAPTURE.with(|slot| -> TestResult<()> {
        let mut capture = slot.try_borrow_mut()?;
        if capture.is_some() {
            return Err("materialized body observation cannot reenter".into());
        }
        *capture = Some(Capture {
            targets: targets.to_vec(),
            bodies: Vec::with_capacity(count),
            instances,
            count,
        });
        Ok(())
    })?;
    let _scope = Scope;
    let prepared = prepare()?;
    let capture = CAPTURE
        .with(|slot| slot.borrow_mut().take())
        .ok_or("materialized observer disappeared")?;
    if capture.bodies.len() != count {
        return Err("actual materialized target was not proved".into());
    }
    Ok((prepared, capture.bodies))
}
pub(crate) fn record(body: &Body) -> Result<(), &'static str> {
    CAPTURE.with(|slot| {
        let mut slot = slot.try_borrow_mut().map_err(|_| "materialized observer is reentrant")?;
        let Some(capture) = slot.as_mut() else { return Ok(()); };
        let Owner::Function { definition, .. } = body.owner else { return Ok(()); };
        if !capture.targets.contains(&definition) { return Ok(()); }
        if capture.bodies.iter().any(|prior| matches!(prior.owner, Owner::Function { definition: actual, .. } if actual == definition) && (!capture.instances || prior.instance == body.instance)) {
            return Err("materialized observer target has multiple proved instances");
        }
        if capture.bodies.len() >= capture.count { return Err("materialized observer exceeded its target bound"); }
        capture.bodies.push(body.clone());
        Ok(())
    })
}
#[test]
fn failed_and_reentered_observation_clears_its_thread_scope() -> TestResult<()> {
    let failed = during::<()>(&[DefId(0)], || {
        assert!(during::<()>(&[], || Ok(())).is_err());
        Err(Error::unlocated("declared observer error"))
    });
    assert!(failed.is_err());
    assert!(CAPTURE.with(|slot| slot.borrow().is_none()));
    let ((), bodies) = during(&[], || Ok(()))?;
    assert!(bodies.is_empty());
    Ok(())
}
