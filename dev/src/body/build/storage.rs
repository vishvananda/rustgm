//! Explicit expression, branch and transfer endpoints retain exact evaluation order.
use super::{BlockId, Builder, Error, NodeId};
use crate::body::{
    scopes,
    storage::{End, Exit, Location, Range},
};

impl Builder<'_> {
    pub(super) fn return_value(
        &mut self,
        node: NodeId,
        value: super::ValueId,
        inner: scopes::Id,
    ) -> Result<(), Error> {
        self.release_return(node, value)?;
        self.exit_scopes(node, Range { inner, outer: None })?;
        self.cleanup(node, 0)?;
        self.terminate(node, super::Terminator::Return(value));
        Ok(())
    }
    pub(super) fn node_scope(&self, node: NodeId) -> Result<scopes::Id, Error> {
        self.scope_plan
            .node(node)
            .map(|n| n.scope)
            .ok_or_else(|| self.error(node, "missing expression scope"))
    }
    pub(super) fn boundary(&self, node: NodeId, kind: scopes::Kind) -> Result<scopes::Id, Error> {
        self.scope_plan
            .boundary(node, kind)
            .ok_or_else(|| self.error(node, "missing storage scope boundary"))
    }
    pub(super) fn single_scope(&self, node: NodeId, scope: scopes::Id) -> Result<Range, Error> {
        let outer = self
            .scope_plan
            .scopes()
            .get(scope.0 as usize)
            .ok_or_else(|| self.error(node, "invalid storage scope"))?
            .parent;
        Ok(Range {
            inner: scope,
            outer,
        })
    }
    pub(super) fn boundary_range(&self, node: NodeId, kind: scopes::Kind) -> Result<Range, Error> {
        self.single_scope(node, self.boundary(node, kind)?)
    }
    pub(super) fn expression_exit(&self, node: NodeId) -> Option<Range> {
        let facts = &self.scope_plan;
        let info = facts.node(node)?;
        let scope = facts.scopes()[info.scope.0 as usize];
        (scope.source == node
            && matches!(
                scope.kind,
                scopes::Kind::Statement | scopes::Kind::Tail | scopes::Kind::Condition
            ))
        .then_some(Range {
            inner: info.scope,
            outer: scope.parent,
        })
    }
    pub(super) fn exit_scopes(&mut self, node: NodeId, scopes: Range) -> Result<(), Error> {
        if let Some(block) = self.current {
            let before = self.index(node, self.body.statements.len())?;
            self.record_exit(node, Location::At { block, before }, scopes)?;
        }
        Ok(())
    }
    pub(super) fn exit_on_edge(
        &mut self,
        node: NodeId,
        target: BlockId,
        scopes: Range,
    ) -> Result<(), Error> {
        if let Some(block) = self.current {
            self.record_exit(node, Location::Edge { block, target }, scopes)?;
        }
        Ok(())
    }
    fn record_exit(
        &mut self,
        node: NodeId,
        location: Location,
        scopes: Range,
    ) -> Result<(), Error> {
        if scopes.outer == Some(scopes.inner) {
            return Ok(());
        }
        self.index(node, self.scope_exits.len())?;
        self.scope_exits.push(Exit {
            location,
            end: End::Scopes(scopes),
        });
        Ok(())
    }
    pub(super) fn end_initializer(&mut self, node: NodeId, start: u32) -> Result<(), Error> {
        if let Some(block) = self.current {
            let before = self.index(node, self.body.statements.len())?;
            let end = self.index(node, self.body.temporaries.len())?;
            self.index(node, self.scope_exits.len())?;
            self.scope_exits.push(Exit {
                location: Location::At { block, before },
                end: End::Temporaries { start, end },
            });
        }
        Ok(())
    }
}
