//! A declaration body yields exact nominal dependencies while retaining its inference graph.
use super::{Checker, Error, ModuleId, NodeId, NodeKey, Scratch, State, TypeId, Work};
use crate::types::ItemId;
use crate::{resolve::model::DefId, sema::model::InitializerTyping};

pub(in crate::sema) struct Dependency {
    pub requirement: Requirement,
    pub source: NodeKey,
}
pub(in crate::sema) enum Requirement {
    Nominal(ItemId),
    Definition(DefId),
}
#[derive(Clone, Copy)]
pub(super) enum Completion {
    Inactive,
    Ordinary,
    Anonymous { before: usize },
    Initializer(usize),
}

impl Checker<'_> {
    pub(in crate::sema) fn pending_body(&self) -> bool {
        !matches!(self.state.completion, Completion::Inactive)
    }
    pub(in crate::sema) fn pending_root(&self) -> bool {
        matches!(self.state.completion, Completion::Anonymous { .. })
    }
    pub(in crate::sema) fn restore_body(&mut self, state: State) {
        self.facts.body_work.suspended_slots -= state.slot_capacity();
        self.state = state;
    }
    pub(in crate::sema) fn into_storage(self) -> (Scratch, Option<Box<State>>) {
        if matches!(self.state.completion, Completion::Inactive) {
            (self.into_scratch(), None)
        } else {
            let counts = &mut self.facts.body_work;
            counts.suspensions += 1;
            counts.suspended_slots += self.state.slot_capacity();
            counts.peak_suspended_slots = counts.peak_suspended_slots.max(counts.suspended_slots);
            (Scratch::default(), Some(Box::new(self.state)))
        }
    }
    pub(in crate::sema) fn start_anonymous_constant(
        &mut self,
        key: NodeKey,
        ty: TypeId,
        module: ModuleId,
    ) -> Result<Option<Dependency>, Error> {
        if let Some(&prior) = self.facts.anonymous_constants.get(&key) {
            return if prior == ty {
                Ok(None)
            } else {
                Err(self.error(key.node, "anonymous constant changed its result type"))
            };
        }
        let before = self.facts.lifetime_fact_count();
        self.begin_constant(key, ty, module);
        self.state.completion = Completion::Anonymous { before };
        self.start_body(key.node)?;
        self.advance_body()
    }
    pub(in crate::sema) fn start_initializer(
        &mut self,
        index: usize,
    ) -> Result<Option<Dependency>, Error> {
        let initializer = self.facts.initializers[index];
        match initializer.typing {
            InitializerTyping::Complete => return Ok(None),
            InitializerTyping::Active => {
                return Err(super::Declarations::error(
                    self.report,
                    initializer.value,
                    "cycle in constant initializer body dependencies",
                ));
            }
            InitializerTyping::Unseen => {}
        }
        self.facts.initializers[index].typing = InitializerTyping::Active;
        self.prepare_constant(initializer.value, initializer.ty, initializer.definition)?;
        self.state.completion = Completion::Initializer(index);
        self.start_body(initializer.value.node)?;
        self.advance_body()
    }
    pub(super) fn start_body(&mut self, body: NodeId) -> Result<(), Error> {
        self.state.steps = 0;
        let expected = self.known(body, self.state.result)?;
        self.state.work.push(Work::Enter(body, Some(expected)));
        Ok(())
    }
    pub(super) fn require_complete_body(
        &self,
        dependency: Option<Dependency>,
    ) -> Result<(), Error> {
        if let Some(dependency) = dependency {
            let message = match dependency.requirement {
                Requirement::Nominal(_) => "missing struct member facts",
                Requirement::Definition(_) => "missing completed associated type definition",
            };
            return Err(self.error(dependency.source.node, message));
        }
        Ok(())
    }
    pub(super) fn complete_body(&mut self) {
        self.facts.body_work.steps += self.state.steps;
        match self.state.completion {
            Completion::Ordinary | Completion::Inactive => {}
            Completion::Anonymous { before } => {
                let annotations = self
                    .facts
                    .constant_annotation_bodies
                    .remove(&self.state.body);
                if self.facts.lifetime_fact_count() != before || annotations {
                    self.facts
                        .anonymous_constants
                        .insert(self.state.body, self.state.result);
                }
            }
            Completion::Initializer(index) => {
                self.facts.initializers[index].typing = InitializerTyping::Complete;
                self.facts
                    .dispatch_dependencies
                    .complete(self.facts.initializers[index].definition);
            }
        }
        self.state.completion = Completion::Inactive;
    }
}

impl State {
    fn slot_capacity(&self) -> usize {
        self.terms.capacity() + self.diverges.capacity() + self.locals.capacity()
    }
}
