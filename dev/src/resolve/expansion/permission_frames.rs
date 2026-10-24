//! Frame authentication shares the actual expander and report indexes.
use super::{Body, Definition, Error, ExpansionId, FileId, Invocation, Package};
use crate::resolve::{
    bindings::{
        KnownFeature,
        panic_contracts::{self, Frame, HeaderData},
    },
    model::{DefKind, Resolution},
};
use crate::syntax::{ast::Kind, item::ItemKind};
use std::rc::Rc;

pub(super) struct FrameView<'a> {
    pub package: &'a Package,
    pub definitions: &'a [Definition],
    pub invocations: &'a [Invocation],
    pub facts: &'a Resolution,
}
impl<'a> FrameView<'a> {
    pub(super) fn matches(&self, frame: Frame) -> bool {
        let Some(index) = frame.context.0.checked_sub(1) else {
            return false;
        };
        self.invocations.get(index as usize).is_some_and(|actual| {
            actual.context == frame.context
                && actual.definition == frame.definition
                && actual.node == frame.call
                && actual.original == frame.original
                && actual.parent == frame.parent
        }) && self
            .definitions
            .get(frame.definition)
            .is_some_and(|actual| {
                !matches!(actual.body, Body::ScopedDeclaration(_))
                    && !matches!(actual.body, Body::Builtin(role) if role.declaration_only())
                    && actual.node == frame.owner
                    && actual.module == frame.module
                    && actual.edition == frame.edition
                    && actual.edition_panic == frame.skip
            })
    }
    pub(super) fn definition(&self, index: usize) -> Result<&'a Definition, Error> {
        let error = |message| Error::at(FileId(0), 0, message);
        let definition = self
            .definitions
            .get(index)
            .ok_or_else(|| error("unknown panic macro definition"))?;
        if matches!(definition.body, Body::Builtin(role) if role.declaration_only()) {
            return Err(error(
                "formatting declaration has no executed expansion frame",
            ));
        }
        if matches!(definition.body, Body::ScopedDeclaration(_)) {
            return Err(error("scoped declaration has no executed expansion frame"));
        }
        let package = self.package;
        let root = self
            .facts
            .crate_root
            .and_then(|root| self.facts.definitions.get(root.0 as usize));
        if root.is_none_or(|root| root.kind != DefKind::Module || root.node.is_some()) {
            return Err(error("macro allowance defining crate root changed"));
        }
        let Some(node) = definition.node else {
            if definition.header.is_some()
                || definition.edition_panic
                || matches!(definition.body, Body::Builtin(role) if role.source_only())
            {
                return Err(error("source panic macro has no defining declaration"));
            }
            return Ok(definition);
        };
        if self
            .package
            .units
            .get(node.file.0 as usize)
            .is_some_and(|unit| {
                unit.configured
                    .ast
                    .nodes
                    .get(node.node.get() as usize)
                    .is_some_and(|actual| {
                        matches!(actual.kind, Kind::Item(index)
                    if unit.configured.ast.items.get(index as usize)
                        .is_some_and(|item| matches!(item.kind, ItemKind::DeclarativeMacro { .. })))
                    })
            })
        {
            return Err(error("scoped declaration has no executed expansion frame"));
        }
        let scope = self
            .facts
            .node_scopes
            .get(&(definition.module, node.node))
            .and_then(|scope| self.facts.scopes.get(scope.0 as usize))
            .ok_or_else(|| error("panic macro has no actual definition scope"))?;
        if scope.module != definition.module
            || package
                .modules
                .get(definition.module.0 as usize)
                .is_none_or(|module| module.file != node.file)
        {
            return Err(error("panic macro definition module changed"));
        }
        let contract = panic_contracts::published(package, node, definition.header.as_ref(), 1)?;
        if contract.node != node
            || contract.edition != definition.edition
            || contract.allows(KnownFeature::EditionPanic) != definition.edition_panic
        {
            return Err(error(
                "panic macro provenance differs from its source contract",
            ));
        }
        if let Body::Builtin(role) = definition.body
            && contract.builtin != Some(role)
        {
            return Err(error("panic macro claimed role changed"));
        }
        Ok(definition)
    }

    pub(super) fn frame(&self, context: ExpansionId) -> Result<Frame, Error> {
        let error = |message| Error::at(FileId(0), 0, message);
        let index = context
            .0
            .checked_sub(1)
            .ok_or_else(|| error("physical source is not an expansion frame"))?;
        self.package
            .source_core()
            .ok_or_else(|| error("missing macro allowance provider"))?
            .macros
            .charge(1 + context.0.checked_ilog2().unwrap_or(0) as usize, 0)
            .map_err(error)?;
        let invocation = self
            .invocations
            .get(index as usize)
            .filter(|invocation| invocation.context == context)
            .ok_or_else(|| error("unknown panic edition frame"))?;
        let unit = self
            .package
            .units
            .get(invocation.node.file.0 as usize)
            .ok_or_else(|| error("unknown panic call-site file"))?;
        if invocation.parent.0 >= context.0
            || !self
                .facts
                .introductions
                .ancestry
                .ancestor(invocation.parent, context)
            || unit
                .configured
                .ast
                .nodes
                .get(invocation.node.node.get() as usize)
                .is_none()
            || unit
                .configured
                .active
                .get(invocation.node.node.get() as usize)
                != Some(&true)
            || invocation.original.start >= invocation.original.end
            || unit
                .source
                .text()
                .get(invocation.original.start as usize..invocation.original.end as usize)
                .is_none()
        {
            return Err(error("invalid panic expansion call site"));
        }
        let definition = self.definition(invocation.definition)?;
        Ok(Frame {
            context,
            definition: invocation.definition,
            call: invocation.node,
            original: invocation.original,
            parent: invocation.parent,
            owner: definition.node,
            module: definition.module,
            edition: definition.edition,
            skip: definition.edition_panic,
        })
    }

    pub(super) fn authenticate_source(
        &self,
        index: usize,
    ) -> Result<Option<Rc<HeaderData>>, Error> {
        let definition = self.definition(index)?;
        let Some(node) = definition.node else {
            return Ok(None);
        };
        let published =
            panic_contracts::published(self.package, node, definition.header.as_ref(), 1)?;
        let header = panic_contracts::checked(self.package, published.source_owner())?
            .ok_or_else(|| Error::at(node.file, 0, "missing actual source macro header"))?;
        if header.summary().node != node {
            return Err(Error::at(
                node.file,
                0,
                "source macro header names another item",
            ));
        }
        let data = header.into_data();
        let matches = match (&definition.header, &data) {
            (None, None) => true,
            (Some(expected), Some(actual)) => Rc::ptr_eq(expected, actual),
            _ => false,
        };
        if !matches {
            return Err(Error::at(
                node.file,
                0,
                "source macro header snapshot changed",
            ));
        }
        Ok(data)
    }
}
