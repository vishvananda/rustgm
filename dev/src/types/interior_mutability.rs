//! Owned-storage mutation is independent of destruction, auto traits and const permissions.
use super::{
    Kind, Store, TypeId,
    aggregate::{Definitions, ShapeReadiness, SharedMutation},
};
use std::collections::HashMap;

/// A conservative type property; it does not describe a particular active value.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Answer {
    Ordinary,
    Interior,
    Unresolved,
}
impl Answer {
    fn combine(self, child: Self) -> Self {
        match (self, child) {
            (Self::Interior, _) | (_, Self::Interior) => Self::Interior,
            (Self::Unresolved, _) | (_, Self::Unresolved) => Self::Unresolved,
            _ => Self::Ordinary,
        }
    }
}
#[derive(Clone, Copy, Default, Debug)]
pub struct Work {
    pub steps: usize,
    pub nodes: usize,
    pub edges: usize,
    pub cache_hits: usize,
}
#[derive(Clone, Copy)]
enum State {
    Active,
    Ready(Answer),
}
struct Frame {
    ty: TypeId,
    start: usize,
    cursor: usize,
    end: usize,
    answer: Answer,
}

/// All cached answers belong to these immutable facts, including pending facts.
/// The borrow prevents registering a policy or completing fields during a query's
/// lifetime. Start a new query after publication; unresolved is never Ordinary.
pub struct Query<'a> {
    types: &'a Store,
    definitions: &'a Definitions,
    answers: HashMap<TypeId, State>,
    frames: Vec<Frame>,
    children: Vec<TypeId>,
    work: Work,
}
impl<'a> Query<'a> {
    #[must_use]
    pub fn new(types: &'a Store, definitions: &'a Definitions) -> Self {
        Self {
            types,
            definitions,
            answers: HashMap::new(),
            frames: Vec::new(),
            children: Vec::new(),
            work: Work::default(),
        }
    }
    #[must_use]
    pub const fn work(&self) -> Work {
        self.work
    }
    /// Inspect owned fields once; references and pointers are terminal slots.
    /// Array element permission is conservative even for zero-length storage;
    /// promotion separately inspects the evaluated value and its active variant.
    /// # Errors
    /// Rejects invalid identities, owned cycles and exhausted graph work. A failed
    /// query is not resumable; pending facts return `Unresolved` without failure.
    pub fn contains(&mut self, ty: TypeId) -> Result<Answer, &'static str> {
        self.enter(ty)?;
        while let Some(frame) = self.frames.last_mut() {
            if frame.cursor == frame.end {
                let frame = self.frames.pop().ok_or("missing mutation query frame")?;
                self.children.truncate(frame.start);
                self.answers.insert(frame.ty, State::Ready(frame.answer));
                continue;
            }
            let child = self.children[frame.cursor];
            self.charge(1)?;
            if let Some(State::Ready(answer)) = self.answers.get(&child).copied() {
                self.work.cache_hits += 1;
                let frame = self.frames.last_mut().ok_or("missing mutation parent")?;
                frame.answer = frame.answer.combine(answer);
                frame.cursor += 1;
            } else {
                self.enter(child)?;
            }
        }
        match self.answers.get(&ty) {
            Some(State::Ready(answer)) => Ok(*answer),
            _ => Err("missing completed mutation answer"),
        }
    }
    fn enter(&mut self, ty: TypeId) -> Result<(), &'static str> {
        self.charge(1)?;
        match self.answers.get(&ty) {
            Some(State::Ready(_)) => {
                self.work.cache_hits += 1;
                return Ok(());
            }
            Some(State::Active) => return Err("owned cycle in shared mutation query"),
            None => {}
        }
        self.work.nodes += 1;
        if self.work.nodes > 1_000_000 {
            return Err("shared mutation query exceeds one million types");
        }
        self.answers.insert(ty, State::Active);
        let start = self.children.len();
        if let Some(answer) = self.inspect(ty)? {
            self.answers.insert(ty, State::Ready(answer));
        } else {
            self.frames.push(Frame {
                ty,
                start,
                cursor: start,
                end: self.children.len(),
                answer: Answer::Ordinary,
            });
        }
        Ok(())
    }
    fn inspect(&mut self, ty: TypeId) -> Result<Option<Answer>, &'static str> {
        match self.types.kind(ty).ok_or("invalid shared mutation type")? {
            Kind::Never
            | Kind::Primitive(_)
            | Kind::Pointer { .. }
            | Kind::Reference { .. }
            | Kind::Function { .. }
            | Kind::FunctionItem { .. } => return Ok(Some(Answer::Ordinary)),
            Kind::Parameter(_) | Kind::Projection(_) | Kind::Opaque(..) | Kind::Dynamic { .. } => {
                return Ok(Some(Answer::Unresolved));
            }
            Kind::Array { element, .. }
            | Kind::ArrayTerm { element, .. }
            | Kind::Slice(element) => self.child(element)?,
            Kind::Closure { environment, .. } => self.child(environment)?,
            Kind::Tuple(fields) => self.fields(fields)?,
            Kind::Adt(item, _) => match self.definitions.shared_mutation(item) {
                None => return Ok(Some(Answer::Unresolved)),
                Some(SharedMutation::Interior) => return Ok(Some(Answer::Interior)),
                Some(SharedMutation::Ordinary) => {
                    match self.definitions.try_shape(self.types, ty)? {
                        ShapeReadiness::Pending => return Ok(Some(Answer::Unresolved)),
                        ShapeReadiness::Ready(shape) => {
                            for variant in shape.variants() {
                                self.fields(variant.fields)?;
                            }
                        }
                    }
                }
            },
        }
        Ok(None)
    }
    fn fields(&mut self, fields: super::ListId) -> Result<(), &'static str> {
        for &field in self
            .types
            .list(fields)
            .ok_or("invalid shared mutation fields")?
        {
            self.child(field)?;
        }
        Ok(())
    }
    fn child(&mut self, ty: TypeId) -> Result<(), &'static str> {
        self.charge(1)?;
        self.work.edges += 1;
        if self.work.edges > 1_000_000 {
            return Err("shared mutation query exceeds one million owned edges");
        }
        self.children.push(ty);
        Ok(())
    }
    fn charge(&mut self, count: usize) -> Result<(), &'static str> {
        self.work.steps = self
            .work
            .steps
            .checked_add(count)
            .filter(|&steps| steps <= 8_000_000)
            .ok_or("shared mutation query work budget exceeded")?;
        Ok(())
    }
}

#[cfg(test)]
#[path = "../../tests/invariants/interior_mutability.rs"]
mod tests;
