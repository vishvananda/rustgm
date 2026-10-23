//! Check reference formation for every body, before demand erases unused code.
use super::Error;
use crate::{
    body::ir::{Action, Body, Place, Program, ProjectionId, Rvalue},
    sema::model::Declarations,
    types::TypeId,
};

/// Borrow and guard-view alignment is independent of loan liveness. An unused
/// reference still has to point at storage aligned for its complete referent.
/// # Errors
/// Rejects packed-field references or inconsistent selected places/layouts.
pub fn check(facts: &mut Declarations, program: &Program) -> Result<(), Error> {
    let mut retained = crate::support::hash::HashSet::default();
    for obligation in &facts.reference_alignment {
        let error = |message| Error {
            node: Some(obligation.source.into()),
            message,
        };
        let required = facts
            .layouts
            .reference_align(&facts.types, &facts.aggregates, obligation.ty)
            .map_err(|e| error(e.message))?;
        if required > obligation.packed {
            return Err(error(if obligation.guard {
                "guard binding would reference an unaligned packed field"
            } else {
                "reference to an unaligned packed field"
            }));
        }
        retained.insert(crate::body::ir::ProofSite::from(obligation.source));
    }
    for body in &program.bodies {
        if body.guards.is_empty()
            && !body
                .values
                .iter()
                .any(|v| matches!(v.kind, Rvalue::Borrow { .. }))
        {
            continue;
        }
        let mut check = Check {
            facts,
            body,
            projections: vec![None; body.projections.len()],
            work: Vec::new(),
        };
        for statement in &body.statements {
            if retained.contains(&body.diagnostic_site(statement.source)) {
                continue;
            }
            let (place, message) = match statement.action {
                Action::Evaluate(id) => match body.values[id.0 as usize].kind {
                    Rvalue::Borrow { place, .. } => {
                        (place, "reference to an unaligned packed field")
                    }
                    _ => continue,
                },
                Action::BindView { place, .. } => (
                    place,
                    "guard binding would reference an unaligned packed field",
                ),
                _ => continue,
            };
            let error = |message| Error {
                node: Some(body.diagnostic_site(statement.source)),
                message,
            };
            let ty = check.place_type(place).map_err(error)?;
            let required = check.required(ty).map_err(error)?;
            if check.available(place).map_err(error)? < required {
                return Err(error(message));
            }
        }
    }
    Ok(())
}
struct Check<'a> {
    facts: &'a mut Declarations,
    body: &'a Body,
    projections: Vec<Option<u32>>,
    work: Vec<ProjectionId>,
}
impl Check<'_> {
    fn required(&mut self, ty: TypeId) -> Result<u32, &'static str> {
        self.facts
            .layouts
            .reference_align(&self.facts.types, &self.facts.aggregates, ty)
            .map_err(|e| e.message)
    }
    fn place_type(&self, place: Place) -> Result<TypeId, &'static str> {
        match place {
            Place::Local(id) => self.body.locals.get(id.0 as usize).map(|p| p.ty),
            Place::Temporary(id) => self.body.values.get(id.0 as usize).map(|p| p.ty),
            Place::Projection(id) => self.body.projections.get(id.0 as usize).map(|p| p.ty),
            Place::Static(id) => self.facts.definitions.get(id.0 as usize).copied().flatten(),
        }
        .ok_or("missing reference alignment place type")
    }
    fn available(&mut self, mut place: Place) -> Result<u32, &'static str> {
        self.work.clear();
        let mut align = loop {
            let Place::Projection(id) = place else {
                break self.required(self.place_type(place)?)?;
            };
            if let Some(align) = self.projections[id.0 as usize] {
                break align;
            }
            let projection = &self.body.projections[id.0 as usize];
            if projection.kind.is_dereference() {
                let align = self.required(projection.ty)?;
                self.projections[id.0 as usize] = Some(align);
                break align;
            }
            if let Place::Projection(base) = projection.base
                && base.0 >= id.0
            {
                return Err("reference alignment projection is not acyclic");
            }
            self.work.push(id);
            place = projection.base;
        };
        while let Some(id) = self.work.pop() {
            align = align.min(self.required(self.body.projections[id.0 as usize].ty)?);
            self.projections[id.0 as usize] = Some(align);
        }
        Ok(align)
    }
}
