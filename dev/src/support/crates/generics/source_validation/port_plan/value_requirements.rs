//! Value typing retains actual source ports and derives requests without paying them.
use super::{
    OperationPort, OperationSite, RequiredKind, Work, g,
    matching::Opening,
    nodes::{Node, Rewrite},
    ports::Ports,
    requests::Builder,
    s,
};

impl Builder<'_, '_, '_> {
    pub(super) fn typed_value(
        &mut self,
        original: &s::Original,
        ports: &Ports,
        ordinal: u32,
        value: s::Value,
        work: &mut Work,
    ) -> Result<(), &'static str> {
        let site = ports.site(ordinal, work)?;
        let port = OperationPort::Value {
            body: original.body,
            value: ordinal,
        };
        self.formation(original, port, site, value.ty, work)?;
        match value.kind {
            s::ValueKind::Unit => self.unit(original, value.ty, work),
            s::ValueKind::Read(place) => {
                let expected = Ports::place_type(original, place, work)?;
                self.exact_relation(original, port, site, value.ty, expected, work)?;
                let node = self.node(original, value.ty, work)?;
                self.coverage.copy(original.body, node, work)?;
                let copy = self.domains.headers.rows().body_inputs[original.body.0 as usize]
                    .builtins
                    .as_ref()
                    .and_then(|b| b.source_copy.as_ref())
                    .ok_or("source Read lost its active Copy query carrier")?;
                self.emit(
                    port,
                    s::InputSite::Body(site),
                    RequiredKind::CopyQuery {
                        ty: value.ty,
                        declaration: copy.declaration,
                    },
                    work,
                )
            }
            s::ValueKind::Borrow(place) => self.borrow_input(
                original,
                ports,
                OperationSite { port, site },
                value.ty,
                place,
                work,
            ),
            s::ValueKind::Constrain(operand) => {
                let actual = original.values[operand as usize].ty;
                let actual_node = self.node(original, actual, work)?;
                let expected_node = self.node(original, value.ty, work)?;
                self.coverage.relation(
                    original.body,
                    s::InputSite::Body(site),
                    actual_node,
                    expected_node,
                    work,
                )?;
                self.arena.same(
                    self.domains,
                    actual_node,
                    expected_node,
                    Opening::Relation {
                        body: original.body,
                    },
                    work,
                )?;
                self.emit(
                    port,
                    s::InputSite::Body(site),
                    RequiredKind::Relation {
                        actual,
                        expected: value.ty,
                    },
                    work,
                )
            }
            s::ValueKind::Function(application) => {
                let node = self.node(original, value.ty, work)?;
                if self.arena.nodes[node.0 as usize] != Node::FunctionItem(application) {
                    return Err("source Function operation has another actual application type");
                }
                Ok(())
            }
            s::ValueKind::Call {
                signature,
                arguments,
                ..
            } => self.call_inputs(
                original,
                ports,
                OperationSite { port, site },
                signature,
                arguments,
                work,
            ),
        }
    }
    fn borrow_input(
        &mut self,
        original: &s::Original,
        ports: &Ports,
        location: OperationSite,
        ty: g::TypeRef,
        place: s::Place,
        work: &mut Work,
    ) -> Result<(), &'static str> {
        let OperationSite { port, site } = location;
        let rows = self.domains.headers.rows();
        let s::Type::Reference {
            mutable,
            region,
            referent,
        } = rows.types[ty.0 as usize]
        else {
            return Err("source Borrow has no actual reference result");
        };
        let actual = Ports::place_type(original, place, work)?;
        if self.node(original, actual, work)? != self.node(original, referent, work)?
            || mutable && !ports.writable(original, place, work)?
        {
            return Err("source Borrow changed its actual referent or writable place");
        }
        let s::Region::Variable { body, slot } = rows.regions[region.0 as usize] else {
            return Err("source Borrow did not retain its actual owned allocation");
        };
        if body != original.body
            || original.variables[slot as usize].allocation
                != (s::RegionAllocation::Inference {
                    needs_provenance: true,
                })
        {
            return Err("source Borrow lost its actual provenance allocation");
        }
        self.coverage.variable(body, slot, true, work)?;
        self.emit(
            port,
            s::InputSite::Body(site),
            RequiredKind::Variable {
                slot,
                needs_provenance: true,
            },
            work,
        )?;
        if matches!(place, s::Place::Local(_)) {
            self.coverage.local_borrow(body, region, work)?;
            self.emit(
                port,
                s::InputSite::Body(site),
                RequiredKind::LocalBorrow(region),
                work,
            )?;
        }
        // Both reference mutability and this exact region remain in Formation;
        // no loan/provenance fact is inferred from successful raw shape checks.
        super::uses::region(self.domains, &self.arena, region, Rewrite::Original, work)?;
        Ok(())
    }
    fn call_inputs(
        &mut self,
        original: &s::Original,
        ports: &Ports,
        location: OperationSite,
        signature: g::TypeRef,
        arguments: g::Range,
        work: &mut Work,
    ) -> Result<(), &'static str> {
        let OperationSite { port, site } = location;
        let OperationPort::Value { value, .. } = port else {
            return Err("source Call request has no actual value port");
        };
        let request = ports.requested[value as usize]
            .and_then(|index| original.calls.get(index as usize))
            .ok_or("source Call omitted its actual retained request")?;
        let rows = self.domains.headers.rows();
        let s::Type::Function(signature) = rows.types[signature.0 as usize] else {
            return Err("source Call type changed from its checked complete signature");
        };
        for (&argument, &expected) in arguments
            .slice(&original.arguments)?
            .iter()
            .zip(signature.parameters.slice(&rows.roots)?)
        {
            self.exact_relation(
                original,
                port,
                site,
                original.values[argument as usize].ty,
                expected,
                work,
            )?;
        }
        self.exact_relation(
            original,
            port,
            site,
            original.values[value as usize].ty,
            signature.result,
            work,
        )?;
        self.emit(
            port,
            s::InputSite::Body(site),
            RequiredKind::Call {
                target: request.target,
                method: request.method,
                effect: request.effect,
            },
            work,
        )
    }
}
