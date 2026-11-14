//! One paid full-type frontier precedes eligibility flow and owns its answers.
use super::{index::Partition, parts::Parts, picked::Selected};
use crate::{
    body::{defining_cleanups::Qualification, ir::Body},
    borrow::{ownership::Paths, regions},
    support::{hash::HashMap, retained::Growth},
    types::{Store, TypeId, aggregate::Definitions, destruction::Qualif},
};

enum Stage {
    Asking,
    Answered,
    Applied,
}
pub(super) struct Queries {
    pub coordinates: Vec<TypeId>,
    questions: Vec<TypeId>,
    index: HashMap<TypeId, usize>,
    answers: Vec<Qualif>,
    masks: Vec<bool>,
    stage: Stage,
}
impl Partition {
    pub(super) fn prepare_queries(
        &mut self,
        types: &mut Store,
        definitions: &Definitions,
        body: &Body,
        paths: &Paths,
    ) -> Result<&[TypeId], &'static str> {
        if self.queries.is_some() {
            return Err("const qualification requests were repeated");
        }
        let selected = Selected::new(&self.tags, paths, body)?;
        let coordinates = selected.coordinates(
            types,
            definitions,
            body,
            &self.tags.parts,
            &self.transfers.candidates,
        )?;
        let mut queries = Queries {
            coordinates,
            questions: Vec::new(),
            index: HashMap::default(),
            answers: Vec::new(),
            masks: Vec::new(),
            stage: Stage::Asking,
        };
        for ordinal in 0..queries.coordinates.len() {
            queries.push(&self.tags.parts, queries.coordinates[ordinal])?;
        }
        // Every genuine cleanup owner can be queried even after a whole move.
        // Root types are actual full body applications, never geometry fields.
        for path in &paths.entries {
            self.tags.parts.charge(1)?;
            if path.parent.is_none() {
                let place = match path.root {
                    crate::borrow::ownership::Root::Local(id) => crate::body::ir::Place::Local(id),
                    crate::borrow::ownership::Root::Temporary(id) => {
                        crate::body::ir::Place::Temporary(id)
                    }
                };
                if body.place_type(place) != Some(path.ty) {
                    return Err("qualification root changed its full body type");
                }
                queries.push(&self.tags.parts, path.ty)?;
            }
        }
        for term in &self.transfers.terms.values {
            match *term {
                super::qterm::Term::Type(ty) => queries.push(&self.tags.parts, ty)?,
                super::qterm::Term::Parameter { root, projected } => {
                    queries.push(&self.tags.parts, root)?;
                    queries.push(&self.tags.parts, projected)?;
                }
                _ => {}
            }
        }
        for row in self.transfers.rows.values() {
            if let Some((_, ty)) = row.expose {
                queries.push(&self.tags.parts, ty)?;
            }
        }
        self.tags.parts.pay(Growth {
            bytes: size_of::<Queries>(),
            work: 1,
        })?;
        self.queries = Some(Box::new(queries));
        Ok(&self
            .queries
            .as_deref()
            .ok_or("const qualification requests disappeared")?
            .questions)
    }
    pub(super) fn accept_queries(
        &mut self,
        body: &Body,
        types: &Store,
        definitions: &Definitions,
        regions: &regions::Body,
        answers: &Qualification<'_>,
    ) -> Result<(), &'static str> {
        let queries = self
            .queries
            .as_deref_mut()
            .ok_or("const qualification has no actual requests")?;
        if !matches!(queries.stage, Stage::Asking) {
            return Err("const qualification answers were already consumed");
        }
        let values = answers.values(body, types, definitions, regions.builtins.as_deref())?;
        self.tags.parts.charge(values.len())?;
        if values.len() != queries.questions.len()
            || values
                .iter()
                .zip(&queries.questions)
                .any(|((actual, _), expected)| actual != expected)
        {
            return Err("const qualification changed its exact full-type request frontier");
        }
        self.tags
            .parts
            .pay(Growth::vector::<Qualif>(0, 0, values.len()))?;
        queries
            .answers
            .try_reserve_exact(values.len())
            .map_err(|_| "const qualification answer allocation failed")?;
        queries.answers.extend(values.iter().map(|&(_, bad)| bad));
        self.tags
            .parts
            .pay(Growth::vector::<bool>(0, 0, self.tags.nodes.len()))?;
        queries
            .masks
            .try_reserve_exact(self.tags.nodes.len())
            .map_err(|_| "qualification filter allocation failed")?;
        for (index, node) in self.tags.nodes.iter().enumerate() {
            self.tags.parts.charge(1)?;
            let mask = if let Some(parent) = node.parent {
                *queries
                    .masks
                    .get(parent.0 as usize)
                    .ok_or("qualification projection parent is missing")?
                    && queries.may_raw(
                        *queries
                            .coordinates
                            .get(index)
                            .ok_or("qualification full field type is missing")?,
                    )?
            } else {
                // Whole-root Destruct qualification cannot discharge its own
                // cleanup. Actual NoDrop (including scoped Copy) needs none.
                queries.answer_raw(
                    *queries
                        .coordinates
                        .get(index)
                        .ok_or("qualification full root type is missing")?,
                )? != Qualif::NoDrop
            };
            queries.masks.push(mask);
        }
        queries.stage = Stage::Answered;
        Ok(())
    }
}
impl Queries {
    fn push(&mut self, parts: &Parts, ty: TypeId) -> Result<(), &'static str> {
        parts.charge(1)?;
        if self.index.contains_key(&ty) {
            return Ok(());
        }
        parts.pay(Growth::map::<TypeId, usize>(
            self.index.len(),
            self.index.capacity(),
            1,
        ))?;
        parts.pay(Growth::vector::<TypeId>(
            self.questions.len(),
            self.questions.capacity(),
            1,
        ))?;
        self.index
            .try_reserve(1)
            .map_err(|_| "const qualification type index allocation failed")?;
        self.questions
            .try_reserve(1)
            .map_err(|_| "const qualification request allocation failed")?;
        self.index.insert(ty, self.questions.len());
        self.questions.push(ty);
        Ok(())
    }
    fn answer_raw(&self, ty: TypeId) -> Result<Qualif, &'static str> {
        let index = self
            .index
            .get(&ty)
            .copied()
            .ok_or("qualification has no exact type request")?;
        self.answers
            .get(index)
            .copied()
            .ok_or("qualification answer is missing")
    }
    fn may_raw(&self, ty: TypeId) -> Result<bool, &'static str> {
        Ok(self.answer_raw(ty)? == Qualif::MayQualify)
    }
    pub(super) fn may(&self, ty: TypeId) -> Result<bool, &'static str> {
        if !matches!(self.stage, Stage::Applied) {
            return Err("qualification flow requires completed scoped answers");
        }
        self.may_raw(ty)
    }
    pub(super) fn mask(&self, coordinate: super::tags::Coordinate) -> Result<bool, &'static str> {
        if !matches!(self.stage, Stage::Applied) {
            return Err("qualification projection requires completed answers");
        }
        self.masks
            .get(coordinate.0 as usize)
            .copied()
            .ok_or("qualification projection filter is missing")
    }
    pub(super) fn apply(&mut self) -> Result<(), &'static str> {
        if !matches!(self.stage, Stage::Answered) || self.answers.len() != self.questions.len() {
            return Err("const qualification is unfinished or already applied");
        }
        self.stage = Stage::Applied;
        Ok(())
    }
}
