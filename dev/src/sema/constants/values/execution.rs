//! Phase-neutral execution requests retain semantic identity across body readiness.
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
    Function(instance::Instance),
    Initializer(DefId),
    Associated(Instance),
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
    FunctionBody(instance::Instance),
    InitializerBody(DefId),
    AssociatedBody(Instance),
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
    pub peak_frames: usize,
    pub peak_slots: usize,
    pub code_bytes: usize,
    pub template_bytes: usize,
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
