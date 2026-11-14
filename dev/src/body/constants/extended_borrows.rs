//! Lexical extension gives storage duration without mutable temporary permission.
use super::invocations::Work;
use crate::{
    body::{
        ir::{Body, Place, ProjectionKind, ValueId},
        scopes::Lifetime,
    },
    support::hash::HashSet,
};

#[derive(Default)]
pub(super) struct Extended {
    values: Option<HashSet<ValueId>>,
    projections: Vec<Option<bool>>,
}
impl Extended {
    pub(super) fn check(
        &mut self,
        body: &Body,
        mut place: Place,
        work: &mut Work,
    ) -> Result<(), &'static str> {
        if self.values.is_none() {
            work.charge(body.temporaries.len())?;
            self.values = Some(
                body.temporaries
                    .iter()
                    .filter(|t| t.lifetime == Lifetime::Static)
                    .map(|t| t.value)
                    .collect(),
            );
        }
        let values = self.values.as_ref().ok_or("missing extension facts")?;
        if values.is_empty() {
            return Ok(());
        }
        if self.projections.len() != body.projections.len() {
            work.charge(body.projections.len())?;
            self.projections.resize(body.projections.len(), None);
        }
        let mut pending = Vec::new();
        let extended = loop {
            work.charge(1)?;
            match place {
                Place::Temporary(id) => break values.contains(&id),
                Place::Local(_) | Place::Static(_) => break false,
                Place::Projection(id) => {
                    if let Some(answer) = self.projections.get(id.0 as usize).copied().flatten() {
                        break answer;
                    }
                    let projection = body
                        .projections
                        .get(id.0 as usize)
                        .ok_or("extended borrow has no typed projection")?;
                    // A dereference borrows the separately owned referent, not
                    // the temporary pointer's storage.
                    if matches!(
                        projection.kind,
                        ProjectionKind::Dereference { .. } | ProjectionKind::RawDereference { .. }
                    ) {
                        self.projections[id.0 as usize] = Some(false);
                        break false;
                    }
                    if pending.len() >= body.projections.len() {
                        return Err("extended borrow has cyclic storage projections");
                    }
                    pending.push(id);
                    place = projection.base;
                }
            }
        };
        for id in pending {
            self.projections[id.0 as usize] = Some(extended);
        }
        if extended {
            return Err("extended initializer temporaries cannot be borrowed mutably");
        }
        Ok(())
    }
}
