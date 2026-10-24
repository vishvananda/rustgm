//! Only the later completed-region consumer can inspect projected cleanup demands.
use super::{Body, Store, regions};
use crate::body::defining_cleanups::{Prepared, State, Work};

/// A transient borrow; no proof or permission is retained in a body or cache.
pub(crate) struct Formed<'a> {
    prepared: &'a Prepared,
}
impl<'a> Formed<'a> {
    pub(crate) fn new(
        state: &'a State,
        body: &'a Body,
        proof: &'a regions::Body,
        work: &mut Work,
    ) -> Result<Self, &'static str> {
        work.charge(4)?;
        if proof.key != body.key() || proof.source != body.source || proof.instance != body.instance
        {
            return Err("defining formation proof belongs to another actual original");
        }
        let builtins = proof
            .builtins
            .as_deref()
            .ok_or("defining formation proof has no retained carrier")?;
        let actual = builtins
            .defining()
            .ok_or("defining formation proof lost its required carrier")?;
        if !std::ptr::eq(actual, state) {
            return Err("defining formation proof retained another actual carrier");
        }
        let prepared = state.checked(body, builtins.environment.as_ref(), work)?;
        Ok(Self { prepared })
    }
    pub(crate) fn check(
        &self,
        types: &Store,
        ty: crate::types::TypeId,
        work: &mut Work,
    ) -> Result<(), &'static str> {
        // Both declaration forms are usable only after this view's actual proof check.
        self.prepared.requirement(types, ty, work).map(|_| ())
    }
}
