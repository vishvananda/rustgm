//! Demand const body typing through the existing resumable declaration worklist.
use super::{Checker, DefId, Error, FileId, Work};
use crate::support::hash::{HashMap, HashSet};
use crate::{
    sema::{bodies, values::execution::Requirement},
    types::Kind,
};

#[cfg(test)]
#[path = "../../../tests/invariants/provider_execution.rs"]
mod tests;

#[derive(Clone, Copy)]
pub(super) enum Request {
    Constant(Requirement),
    SourceBody(DefId),
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Phase {
    Begin,
    Prepared,
    Resume,
}

#[derive(Default)]
pub(super) struct Pending {
    active: HashSet<DefId>,
    suspended: HashMap<DefId, Box<bodies::State>>,
}

/// Header/WF preparation plus explicit current bodies; unrelated bodies stay Pending.
pub(crate) fn analyze_local_roots(
    report: &crate::resolve::expansion::Report,
    roots: &[DefId],
    executor: Box<dyn crate::sema::values::execution::Executor>,
) -> Result<crate::sema::model::Declarations, Error> {
    if roots.is_empty() {
        return Err(Error::unlocated(
            "selected local runtime requires at least one actual root",
        ));
    }
    let mut checker = Checker::new(report)?;
    checker.facts.set_constant_executor(executor);
    let result = (|| {
        checker.run()?;
        crate::sema::compiler_items::pointee::authenticate(report, &checker.facts)?;
        let mut steps = 0;
        let limit = checker.work_limit();
        for &definition in roots {
            checker.work.push(Work::Execution(
                Request::SourceBody(definition),
                Phase::Begin,
            ));
            checker.drain(&mut steps, limit)?;
        }
        // Source selection does not excuse initializer WF or const requirements.
        for index in 0..checker.facts.initializers.len() {
            if !checker.facts.initializers[index].type_checked() {
                let definition = checker.facts.initializers[index].definition;
                checker.work.push(Work::Execution(
                    Request::Constant(Requirement::InitializerBody(definition)),
                    Phase::Begin,
                ));
                checker.drain(&mut steps, limit)?;
            }
        }
        crate::sema::compiler_items::pointee::authenticate(report, &checker.facts)?;
        Ok(())
    })();
    result.map_err(|error| {
        checker
            .facts
            .retain_error_warnings(report.diagnostic(error))
    })?;
    Ok(checker.facts)
}

impl Checker<'_> {
    pub(super) fn execution_body(&mut self, request: Request, phase: Phase) -> Result<(), Error> {
        let owner = match request {
            Request::Constant(requirement) => self.execution_owner(requirement, phase)?,
            Request::SourceBody(definition) => Some((self.source_body_owner(definition)?, None)),
        };
        let Some((definition, initializer)) = owner else {
            return Ok(());
        };
        if initializer.map_or_else(
            || self.facts.typed_function_bodies.contains(&definition),
            |index| self.facts.initializers[index].type_checked(),
        ) {
            return Ok(());
        }
        if phase == Phase::Begin && !self.execution_bodies.active.insert(definition) {
            return Err(Error::at(
                FileId(0),
                0,
                "cycle in const function body declaration dependencies",
            ));
        }
        if phase == Phase::Begin && initializer.is_none() && matches!(request, Request::Constant(_))
        {
            let function = self
                .facts
                .function_indices
                .get(&definition)
                .and_then(|&i| self.facts.functions.get(i))
                .copied()
                .ok_or_else(|| Error::at(FileId(0), 0, "missing demanded function annotations"))?;
            if let Some(body) = function.body {
                self.work.push(Work::Execution(request, Phase::Prepared));
                let context = self.local_context(function.node, super::regions::Context::ALIAS);
                self.constant_dependencies_in(body, context)?;
                return Ok(());
            }
        }
        if phase == Phase::Prepared
            && (!self.execution_bodies.active.contains(&definition)
                || self.execution_bodies.suspended.contains_key(&definition))
        {
            return Err(Error::at(
                FileId(0),
                0,
                "prepared const body has no exact pending owner",
            ));
        }
        let mut scratch = self.constant_scratch.take_typing();
        scratch.resumable();
        let mut checker = bodies::Checker::with_scratch(self.report, &mut self.facts, scratch);
        checker.availability = match request {
            Request::Constant(_) => {
                crate::sema::traits::implementations::Availability::Declarations
            }
            Request::SourceBody(_) => {
                crate::sema::traits::implementations::Availability::Executable
            }
        };
        let progress = if phase == Phase::Resume {
            let state = self
                .execution_bodies
                .suspended
                .remove(&definition)
                .ok_or_else(|| Error::at(FileId(0), 0, "missing suspended const function body"))?;
            checker.restore_body(*state);
            checker.advance_body()?
        } else if let Some(index) = initializer {
            checker.start_initializer(index)?
        } else {
            checker.start_execution_function(definition)?
        };
        if progress.is_none() {
            checker.complete_constant_patterns()?;
        }
        let (scratch, state) = checker.into_storage();
        self.constant_scratch.retain_typing(scratch);
        if let Some(dependency) = progress {
            let state = state.ok_or_else(|| {
                Error::at(FileId(0), 0, "const body dependency has no continuation")
            })?;
            self.execution_bodies.suspended.insert(definition, state);
            self.work.push(Work::Execution(request, Phase::Resume));
            self.work
                .push(Work::body_dependency(dependency.requirement));
        } else {
            self.execution_bodies.active.remove(&definition);
            if initializer.is_none() {
                self.facts.typed_function_bodies.insert(definition);
            }
        }
        Ok(())
    }
    fn source_body_owner(&self, definition: DefId) -> Result<DefId, Error> {
        let declared = self
            .report
            .resolution
            .definitions
            .get(definition.0 as usize)
            .filter(|declared| declared.kind == crate::resolve::model::DefKind::Function)
            .ok_or_else(|| {
                Error::unlocated("selected body lacks its actual function declaration")
            })?;
        let source = declared.node.ok_or_else(|| {
            Error::unlocated("selected current body cannot use an imported original")
        })?;
        let function = self
            .facts
            .function_indices
            .get(&definition)
            .and_then(|&index| self.facts.functions.get(index))
            .ok_or_else(|| {
                self.error(source, "selected body lacks its checked current function")
            })?;
        let crate::syntax::ast::Kind::Item(index) = self.kind(source) else {
            return Err(self.error(source, "selected function lost its actual source item"));
        };
        let crate::syntax::item::ItemKind::Function {
            body: Some(body), ..
        } = self.report.package.unit(source.file).configured.ast.items[index as usize].kind
        else {
            return Err(self.error(
                source,
                "selected current body requires its actual source body",
            ));
        };
        if function.definition != definition
            || function.node != source
            || function.body
                != Some(crate::resolve::unit::NodeKey {
                    node: body,
                    ..source
                })
        {
            return Err(self.error(
                source,
                "selected body changed its actual function/body identity",
            ));
        }
        Ok(definition)
    }
    pub(super) fn selected_bodies(&mut self) -> Result<(), Error> {
        if self.selected.is_none() {
            return Err(Error::unlocated(
                "selected body requires its actual private consumer",
            ));
        }
        let mut steps = 0;
        let limit = self.work_limit();
        #[cfg(test)]
        eprintln!(
            "selected-body-layout work={} constant-request={} request={} checker={} pending={}",
            size_of::<Work>(),
            size_of::<Requirement>(),
            size_of::<Request>(),
            size_of::<Checker<'_>>(),
            size_of::<Pending>()
        );
        for index in 0..self.facts.functions.len() {
            let function = self.facts.functions[index];
            if function.body.is_some() {
                self.work.push(Work::Execution(
                    Request::SourceBody(function.definition),
                    Phase::Begin,
                ));
                self.drain(&mut steps, limit)?;
            }
        }
        Ok(())
    }
    fn execution_owner(
        &mut self,
        requirement: Requirement,
        phase: Phase,
    ) -> Result<Option<(DefId, Option<usize>)>, Error> {
        let owner = match requirement {
            Requirement::DiscriminantValue(_) => {
                return Err(Error::at(
                    FileId(0),
                    0,
                    "enum value readiness must remain in its constant dependency graph",
                ));
            }
            Requirement::AnonymousBody { instance, .. } => {
                return Err(self.error(
                    instance.source(),
                    "anonymous template must complete with its enclosing declaration",
                ));
            }
            Requirement::InlineBody { source, .. } => {
                return Err(self.error(
                    source,
                    "inline constant body must complete with its owning declaration",
                ));
            }
            Requirement::FunctionBody(instance) => {
                let Some(Kind::FunctionItem { item, .. }) = self.facts.types.kind(instance.ty())
                else {
                    return Err(Error::at(
                        FileId(0),
                        0,
                        "constant body requirement lost function identity",
                    ));
                };
                (DefId(item.0), None)
            }
            Requirement::InitializerBody(definition) => {
                let source = self.report.resolution.definitions[definition.0 as usize]
                    .node
                    .ok_or_else(|| {
                        Error::at(FileId(0), 0, "missing constant declaration source")
                    })?;
                if phase == Phase::Begin {
                    if !self.execution_bodies.active.insert(definition) {
                        return Err(self.error(source, "cycle in required initializer bodies"));
                    }
                    self.work.push(Work::Execution(
                        Request::Constant(requirement),
                        Phase::Prepared,
                    ));
                    self.require_initializer_readiness(source, definition)?;
                    return Ok(None);
                }
                let index = self.facts.initializer_index(definition).ok_or_else(|| {
                    self.error(source, "missing required initializer declaration")
                })?;
                (definition, Some(index))
            }
            Requirement::SourceAssociatedBody(source) => {
                let (_, original) = source
                    .identity(&self.facts)
                    .map_err(|e| Error::at(FileId(0), 0, e))?;
                let result = source
                    .result(&self.facts)
                    .map_err(|e| Error::at(FileId(0), 0, e))?;
                let selected = source
                    .resolve(&mut self.facts, result)
                    .map_err(|e| self.error(original, e))?;
                self.require_initializer_readiness(original, selected.selected.definition)?;
                return Ok(None);
            }
            Requirement::AssociatedBody { instance, .. } => {
                let source = self.report.resolution.definitions[instance.definition.0 as usize]
                    .node
                    .ok_or_else(|| Error::at(FileId(0), 0, "missing associated constant source"))?;
                self.require_initializer_readiness(source, instance.definition)?;
                return Ok(None);
            }
        };
        Ok(Some(owner))
    }
}
