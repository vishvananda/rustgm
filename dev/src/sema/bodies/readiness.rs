//! A declaration body yields exact nominal dependencies while retaining its inference graph.
use super::{Checker, Error, ModuleId, NodeId, NodeKey, Scratch, State, TypeId, Work};
use crate::types::ItemId;
use crate::{
    resolve::model::{DefId, DefKind},
    sema::model::InitializerTyping,
};

pub(in crate::sema) struct Dependency {
    pub requirement: Requirement,
    pub source: NodeKey,
}
#[derive(Clone, Copy)]
pub(in crate::sema) enum Requirement {
    Nominal(ItemId),
    Inherent(ItemId, crate::resolve::names::NameId),
    Definition(DefId),
    Constructor(DefId),
    ImplHeader(DefId),
    ImplementationValidated(DefId),
    Fields(ItemId),
    Opaque(crate::types::opaque::OpaqueId),
    SourceResultContracts(DefId),
    CallerLocation(DefId),
}
impl From<crate::sema::traits::selection::Dependency> for Requirement {
    fn from(dependency: crate::sema::traits::selection::Dependency) -> Self {
        use crate::sema::traits::selection::Dependency;
        match dependency {
            Dependency::Header(owner) => Self::ImplHeader(owner),
            Dependency::ImplementationValidated(owner) => Self::ImplementationValidated(owner),
            Dependency::Fields(item) => Self::Fields(item),
            Dependency::Opaque(id) => Self::Opaque(id),
            Dependency::SourceResultContracts(id) => Self::SourceResultContracts(id),
        }
    }
}
#[derive(Clone, Copy)]
pub(super) enum Completion {
    Inactive,
    Ordinary,
    Anonymous { before: usize },
    Initializer(usize),
}

impl Checker<'_> {
    /// Yield only the unresolved value currently being consumed. Header/type
    /// readiness does not request any constant or static initializer value.
    pub(super) fn value_readiness(&self, node: NodeId) -> Option<Requirement> {
        if !matches!(
            self.availability,
            crate::sema::traits::implementations::Availability::Declarations
        ) || !matches!(
            self.kind(node),
            crate::syntax::ast::Kind::Name(..)
                | crate::syntax::ast::Kind::Path { .. }
                | crate::syntax::ast::Kind::Qualified { .. }
        ) {
            return None;
        }
        let crate::resolve::model::Target::Definition(definition) =
            self.facts.node(self.key(node))?.reference?
        else {
            return None;
        };
        if self.facts.definitions.get(definition.0 as usize)?.is_some()
            || self.state.locals[definition.0 as usize].is_some()
        {
            return None;
        }
        match self.report.resolution.definitions[definition.0 as usize].kind {
            DefKind::Function | DefKind::Const | DefKind::Static => {
                Some(Requirement::Definition(definition))
            }
            DefKind::Constructor | DefKind::Variant => Some(Requirement::Constructor(definition)),
            _ => None,
        }
    }
    pub(in crate::sema) fn start_execution_function(
        &mut self,
        definition: DefId,
    ) -> Result<Option<Dependency>, Error> {
        let index = self
            .facts
            .function_indices
            .get(&definition)
            .copied()
            .ok_or_else(|| {
                self.error(
                    NodeId::new(0),
                    "missing demanded const function declaration",
                )
            })?;
        self.start_function(self.facts.functions[index])
    }
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
    pub(super) fn suspend_body(&mut self) -> Box<State> {
        self.state.retain_sparse();
        let counts = &mut self.facts.body_work;
        counts.suspensions += 1;
        counts.suspended_slots += self.state.slot_capacity();
        counts.peak_suspended_slots = counts.peak_suspended_slots.max(counts.suspended_slots);
        let mut scratch = Scratch::default();
        scratch.resumable();
        let fresh = Checker::with_scratch(self.report, self.facts, scratch).state;
        Box::new(std::mem::replace(&mut self.state, fresh))
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
        self.prepare_constant(initializer)?;
        self.state.completion = Completion::Initializer(index);
        self.start_body(initializer.value.node)?;
        self.advance_body()
    }
    pub(super) fn start_body(&mut self, body: NodeId) -> Result<(), Error> {
        self.state.steps = 0;
        let expected = self.result_term(body)?;
        self.state.work.push(Work::CoerceElement(body, expected));
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
                Requirement::Inherent(..) => "missing prepared inherent constructor headers",
                Requirement::Definition(_) => "missing completed associated type definition",
                Requirement::Constructor(_) => "missing completed constructor type",
                Requirement::ImplHeader(_) => "missing prepared implementation header",
                Requirement::ImplementationValidated(_) => "missing validated implementation",
                Requirement::Fields(_) => "missing nominal field declarations",
                Requirement::Opaque(_) => "missing completed opaque hidden type",
                Requirement::SourceResultContracts(_) => "missing checked source Pointee contracts",
                Requirement::CallerLocation(_) => "missing checked source caller-location data",
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
    pub(super) fn retain_sparse(&mut self) {
        self.terms
            .retain_sparse(self.touched.iter().map(|&node| NodeKey {
                file: self.file,
                node,
            }));
        self.diverges
            .retain_sparse(self.touched.iter().map(|node| node.get() as usize));
        self.locals
            .retain_sparse(self.bindings.iter().map(|binding| binding.0 as usize));
    }
    fn slot_capacity(&self) -> usize {
        self.terms.capacity() + self.diverges.capacity() + self.locals.capacity()
    }
}
