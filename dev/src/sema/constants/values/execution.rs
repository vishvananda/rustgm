//! Phase-neutral execution requests retain semantic identity across body readiness.
pub use super::super::computation::SourceComputation;
pub(crate) use super::super::computation::{
    BodyCaller, CallerInstance, DeclarationEpoch, PreparedConstant, caller_namespace_available,
    caller_published, declaration_epoch, reserve_normalization, with_namespace,
    with_registered_namespace,
};
use super::{Constant, Instance};
use crate::{
    resolve::{expansion::Report, model::DefId, package::Error, unit::NodeKey},
    sema::model::Declarations,
    types::{TypeId, instance},
};

/// The selected body identity includes every closed generic argument.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Target {
    Expression {
        source: NodeKey,
        result: TypeId,
        context: Option<Instance>,
    },
    Anonymous {
        instance: crate::sema::inline_constants::InlineInstance,
        result: TypeId,
    },
    Function(instance::Instance),
    Initializer(DefId),
    SourceAssociated {
        source: SourceComputation,
        result: TypeId,
    },
    Associated {
        instance: Instance,
        result: TypeId,
    },
}

/// A value demand is distinct from a declaration's const permission.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Request {
    pub target: Target,
    pub arguments: Vec<Constant>,
}

/// Only the selected dependency is prepared before the retained query resumes.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Requirement {
    SourceAssociatedBody(SourceComputation),
    DiscriminantValue(crate::sema::members::VariantId),
    FunctionBody(instance::Instance),
    InitializerBody(DefId),
    InlineBody {
        source: NodeKey,
        result: TypeId,
    },
    AnonymousBody {
        instance: crate::sema::inline_constants::InlineInstance,
        result: TypeId,
    },
    AssociatedBody {
        instance: Instance,
        result: TypeId,
    },
}

pub enum Progress {
    Complete(Constant),
    Prepare(Requirement),
}
/// Actual interpreter work and charged retained typed storage, never an oracle.
#[derive(Default)]
pub struct Work {
    pub expressions: usize,
    pub frames_started: usize,
    pub steps: usize,
    pub resumes: usize,
    /// Actual source-budget preparation visits retained across required queries.
    pub value_preparation: usize,
    /// Append-time initializer-index publications, never repeated by value queries.
    pub initializer_indexed: usize,
    /// Every shared initializer-index hit or miss, including cached value demands.
    pub initializer_lookups: std::cell::Cell<usize>,
    /// Rows visited by the one ordinary-initializer completion pass.
    pub initializer_visits: usize,
    /// Required ordinary closed initializer demands, including cached values.
    pub initializer_validations: usize,
    /// Discriminant finish entry visits, including every suspended entry resume.
    pub discriminant_visits: usize,
    /// Combined frame/value/cell slots retained by all suspended requests.
    pub peak_suspended_slots: usize,
    pub peak_frames: usize,
    pub peak_slots: usize,
    pub code_bytes: usize,
    pub template_bytes: usize,
    /// Owned checked constant bodies retained before ordinary CFG handoff.
    pub expression_bytes: usize,
    pub anonymous_templates: usize,
    pub anonymous_instances: usize,
    pub associated_templates: usize,
    pub associated_instances: usize,
    /// Conservative retained allocation for typed completed instance values.
    pub associated_value_bytes: usize,
    /// Lazy caller/use catalog allocation, separate from completed-value storage.
    pub source_context_bytes: usize,
}

/// The orchestrator supplies checked CFG execution without a semantic dependency
/// on CFG construction, ownership checking or runtime lowering.
pub trait Executor {
    /// Validate the whole checked expression without executing a branch or call.
    /// # Errors
    /// Rejects const-ineligible operations, including those in skipped branches.
    fn check(
        &mut self,
        report: &Report,
        facts: &mut Declarations,
        target: Target,
    ) -> Result<(), Error>;
    /// Resume this exact request; unfinished frames belong to the implementation.
    /// # Errors
    /// Reports const restrictions, evaluation faults and deterministic limits.
    fn advance(
        &mut self,
        report: &Report,
        facts: &mut Declarations,
        request: &Request,
    ) -> Result<Progress, Error>;
}
