//! Actual raw operations derive complete neutral requests before any acceptance.
use super::{
    OperationPort, RequiredKind, Work, g, nodes::Node, ports::Ports, requests::Builder, s,
};

impl Builder<'_, '_, '_> {
    pub(super) fn operations(
        &mut self,
        original: &s::Original,
        work: &mut Work,
    ) -> Result<(), &'static str> {
        let mut ports = Ports::new(original, &mut self.arena, work)?;
        let result = self.operation_ports(original, &mut ports, work);
        drop(ports);
        self.arena.held_scratch(0, work)?;
        result
    }
    fn operation_ports(
        &mut self,
        original: &s::Original,
        ports: &mut Ports,
        work: &mut Work,
    ) -> Result<(), &'static str> {
        self.entry_types(original, work)?;
        ports.projections(self, original, work)?;
        for (index, local) in original.locals.iter().enumerate() {
            self.formation(
                original,
                OperationPort::Local {
                    body: original.body,
                    local: u32::try_from(index).map_err(|_| "source local ordinal overflow")?,
                },
                local.site,
                local.ty,
                work,
            )?;
        }
        for (index, value) in original.values.iter().enumerate() {
            let ordinal = u32::try_from(index).map_err(|_| "source value ordinal overflow")?;
            self.typed_value(original, ports, ordinal, *value, work)?;
        }
        for (index, statement) in original.statements.iter().enumerate() {
            work.charge(1)?;
            if let s::Action::Store { place, value } = statement.action {
                if matches!(place, s::Place::Projection(_))
                    && !ports.writable(original, place, work)?
                {
                    return Err("source Store writes through an immutable dereference");
                }
                let actual = original.values[value as usize].ty;
                let expected = Ports::place_type(original, place, work)?;
                self.exact_relation(
                    original,
                    OperationPort::Statement {
                        body: original.body,
                        statement: u32::try_from(index)
                            .map_err(|_| "source statement ordinal overflow")?,
                    },
                    statement.site,
                    actual,
                    expected,
                    work,
                )?;
            }
        }
        for (index, block) in original.blocks.iter().enumerate() {
            work.charge(1)?;
            if let g::Terminator::Return(value) = block.terminator {
                self.exact_relation(
                    original,
                    OperationPort::Terminator {
                        body: original.body,
                        block: u32::try_from(index).map_err(|_| "source block ordinal overflow")?,
                    },
                    block.site,
                    original.values[value as usize].ty,
                    original.result,
                    work,
                )?;
            }
        }
        Ok(())
    }
    fn entry_types(&mut self, original: &s::Original, work: &mut Work) -> Result<(), &'static str> {
        let rows = self.domains.headers.rows();
        let header = self.domains.function(original.declaration, work)?;
        let parameters = header.original.parameters.slice(&rows.roots)?;
        for (index, local) in original.locals.iter().enumerate() {
            work.charge(1)?;
            let g::LocalKind::Parameter(formal) = local.kind else {
                continue;
            };
            let actual = *parameters
                .get(formal as usize)
                .ok_or("source input local has no actual original formal")?;
            let port = OperationPort::Parameter {
                body: original.body,
                formal,
                local: u32::try_from(index).map_err(|_| "source parameter local index overflow")?,
            };
            self.normalized_relation(
                original,
                port,
                s::InputSite::Body(local.site),
                actual,
                local.ty,
                work,
            )?;
        }
        for binding in &original.parameter_bindings {
            work.charge(1)?;
            let input = &original.locals[binding.input as usize];
            let bound = &original.locals[binding.local as usize];
            let g::LocalKind::Parameter(formal) = input.kind else {
                return Err("source named input changed its actual formal kind");
            };
            let port = OperationPort::Parameter {
                body: original.body,
                formal,
                local: binding.local,
            };
            self.exact_relation(original, port, bound.site, input.ty, bound.ty, work)?;
        }
        // The full raw normalizations/equalities remain available for reproof.
        // Shape admission never turns one of them into an authenticated alias.
        self.normalized_relation(
            original,
            OperationPort::Declaration {
                body: original.body,
                owner: original.declaration,
            },
            s::InputSite::Declaration {
                owner: original.declaration,
                site: 0,
            },
            header.original.result,
            original.result,
            work,
        )
    }
    fn normalized_relation(
        &mut self,
        original: &s::Original,
        port: OperationPort,
        site: s::InputSite,
        actual: g::TypeRef,
        expected: g::TypeRef,
        work: &mut Work,
    ) -> Result<(), &'static str> {
        let actual_node = self.node(original, actual, work)?;
        let expected_node = self.node(original, expected, work)?;
        self.arena.same(
            self.domains,
            actual_node,
            expected_node,
            super::matching::Opening::Relation {
                body: original.body,
            },
            work,
        )?;
        self.emit(
            port,
            site,
            RequiredKind::Relation { actual, expected },
            work,
        )
    }
    pub(super) fn exact_relation(
        &mut self,
        original: &s::Original,
        port: OperationPort,
        site: u32,
        actual: g::TypeRef,
        expected: g::TypeRef,
        work: &mut Work,
    ) -> Result<(), &'static str> {
        if self.node(original, actual, work)? != self.node(original, expected, work)? {
            return Err("source operation type differs without an explicit supported Constrain");
        }
        self.emit(
            port,
            s::InputSite::Body(site),
            RequiredKind::Relation { actual, expected },
            work,
        )
    }
    pub(super) fn unit(
        &mut self,
        original: &s::Original,
        ty: g::TypeRef,
        work: &mut Work,
    ) -> Result<(), &'static str> {
        let node = self.node(original, ty, work)?;
        if self.arena.nodes[node.0 as usize] != Node::Scalar(crate::support::crates::Scalar::Unit) {
            return Err("source Unit operation changed its actual scalar type");
        }
        Ok(())
    }
}
