//! Value typing retains actual source ports and derives requests without paying them.
use super::{
    Context, OperationPort, OperationSite, RequiredKind, Work, g,
    matching::Opening,
    nodes::{Node, NodeId, Rewrite},
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
            s::ValueKind::Aggregate { fields } => {
                self.nominal_value(original, (value.ty, None), fields, work)
            }
            s::ValueKind::VariantAggregate { variant, fields } => {
                self.nominal_value(original, (value.ty, Some(variant)), fields, work)
            }
            s::ValueKind::IsVariant { place, variant } => {
                self.variant_test(original, (value.ty, place), variant, work)
            }
            s::ValueKind::EnumCast(operand) => self.enum_cast(original, value.ty, operand, work),
            // Domain validation checked the exact primitive type and canonical bits,
            // an assembly effect's unit type and operand types, and a constant
            // read's declared scalar type.
            // Formation above remains an ordinary unpaid request.
            s::ValueKind::Scalar(_)
            | s::ValueKind::ScalarCast(_)
            | s::ValueKind::Binary { .. }
            | s::ValueKind::Text(_)
            | s::ValueKind::Assembly { .. }
            | s::ValueKind::Constant(_) => Ok(()),
            s::ValueKind::Read(place) => self.read_input(
                original,
                OperationSite { port, site },
                value.ty,
                place,
                work,
            ),
            s::ValueKind::Borrow(place) | s::ValueKind::TwoPhaseBorrow(place) => self.borrow_input(
                original,
                ports,
                OperationSite { port, site },
                value.ty,
                place,
                work,
            ),
            s::ValueKind::Constrain(operand) => self.constrained(
                original,
                OperationSite { port, site },
                value.ty,
                operand,
                work,
            ),
            s::ValueKind::Function(application) => {
                let node = self.node(original, value.ty, work)?;
                if self.arena.nodes[node.0 as usize] != Node::FunctionItem(application) {
                    return Err("source Function operation has another actual application type");
                }
                Ok(())
            }
            s::ValueKind::Reify(operand) => self.reified(original, value.ty, operand, work),
            s::ValueKind::Invoke {
                signature,
                arguments,
                ..
            } => self.invoke_inputs(
                original,
                OperationSite { port, site },
                (value.ty, signature),
                arguments,
                work,
            ),
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
            s::ValueKind::PointerCall {
                signature,
                callee,
                arguments,
                ..
            } => self.pointer_inputs(
                original,
                OperationSite { port, site },
                (signature, callee),
                arguments,
                work,
            ),
        }
    }
    /// A constraint relates its operand's actual type to its own expected type
    /// at its site, and requests that relation.
    fn constrained(
        &mut self,
        original: &s::Original,
        location: OperationSite,
        expected: g::TypeRef,
        operand: u32,
        work: &mut Work,
    ) -> Result<(), &'static str> {
        let OperationSite { port, site } = location;
        let actual = original.values[operand as usize].ty;
        let actual_node = self.node(original, actual, work)?;
        let expected_node = self.node(original, expected, work)?;
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
            RequiredKind::Relation { actual, expected },
            work,
        )
    }
    /// Version thirteen: a reification re-proves its operand's applied item
    /// signature against its own pointer type. The operand's Function value
    /// already charged the item's use and formation.
    fn reified(
        &mut self,
        original: &s::Original,
        ty: g::TypeRef,
        operand: u32,
        work: &mut Work,
    ) -> Result<(), &'static str> {
        let s::ValueKind::Function(application) = original.values[operand as usize].kind else {
            return Err("source reification lost its function item or pointer type");
        };
        let Some(&s::Type::Function(signature)) =
            self.domains.headers.rows().types.get(ty.0 as usize)
        else {
            return Err("source reification lost its function item or pointer type");
        };
        let pointer = self.node(original, ty, work)?;
        let caller = Context {
            owner: original.declaration,
            frame: Some(
                self.domains.headers.rows().body_inputs
                    [self.domains.headers.original_index(original.body)?]
                .frame,
            ),
            body: Some(original.body),
        };
        self.arena.reified(
            self.domains,
            application,
            caller,
            (signature, pointer),
            work,
        )
    }
    /// A read relates its exact place type and requests the active Copy query.
    fn read_input(
        &mut self,
        original: &s::Original,
        location: OperationSite,
        ty: g::TypeRef,
        place: s::Place,
        work: &mut Work,
    ) -> Result<(), &'static str> {
        let OperationSite { port, site } = location;
        let expected = Ports::place_type(original, place, work)?;
        self.exact_relation(original, port, site, ty, expected, work)?;
        let node = self.node(original, ty, work)?;
        self.coverage.copy(original.body, node, work)?;
        let copy = self.domains.headers.rows().body_inputs
            [self.domains.headers.original_index(original.body)?]
        .builtins
        .as_ref()
        .and_then(|b| b.source_copy.as_ref())
        .ok_or("source Read lost its active Copy query carrier")?;
        self.emit(
            port,
            s::InputSite::Body(site),
            RequiredKind::CopyQuery {
                ty,
                declaration: copy.declaration,
            },
            work,
        )
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
        let referent = self.node(original, referent, work)?;
        if self.node(original, actual, work)? != referent
            && !self.coerced_reborrow(original, place, referent, work)?
            || mutable && !ports.writable(original, place, work)?
        {
            return Err("source Borrow changed its actual referent or writable place");
        }
        let s::Region::Variable { body, slot } = rows.regions[region.0 as usize] else {
            return Err("source Borrow did not retain its actual owned allocation");
        };
        let allocation = original
            .variables
            .get(slot as usize)
            .filter(|_| body == original.body)
            .map(|variable| variable.allocation);
        let provenance = match allocation {
            Some(s::RegionAllocation::Inference {
                needs_provenance: true,
            }) => true,
            // Version thirteen: an implicit reborrow of a dereferenced place
            // takes an inference region with no borrow-expression provenance.
            // The consumer derives its bounds from its own place walk.
            Some(s::RegionAllocation::Inference {
                needs_provenance: false,
            }) if self.domains.bodies && reborrowed(original, place) => false,
            _ => return Err("source Borrow lost its actual provenance allocation"),
        };
        self.coverage.variable(body, slot, provenance, work)?;
        self.emit(
            port,
            s::InputSite::Body(site),
            RequiredKind::Variable {
                slot,
                needs_provenance: provenance,
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
    /// A call operator relates its untupled inputs and its value to the
    /// caller-side signature; the receiver's own call trait is selected again
    /// by every consumer and contributes no planned relation here.
    fn invoke_inputs(
        &mut self,
        original: &s::Original,
        location: OperationSite,
        (result, signature): (g::TypeRef, g::TypeRef),
        arguments: g::Range,
        work: &mut Work,
    ) -> Result<(), &'static str> {
        let OperationSite { port, site } = location;
        let rows = self.domains.headers.rows();
        let s::Type::Function(signature) = rows.types[signature.0 as usize] else {
            return Err("source call operator type changed from its checked signature");
        };
        let [_, inputs @ ..] = arguments.slice(&original.arguments)? else {
            return Err("source call operator lost its actual receiver");
        };
        let expected = signature.parameters.slice(&rows.roots)?;
        if inputs.len() != expected.len() {
            return Err("source call operator arguments are incomplete");
        }
        for (&argument, &expected) in inputs.iter().zip(expected) {
            self.exact_relation(
                original,
                port,
                site,
                original.values[argument as usize].ty,
                expected,
                work,
            )?;
        }
        self.exact_relation(original, port, site, result, signature.result, work)
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
        self.call_relations(original, location, signature, arguments, work)?;
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
    /// Version thirteen: a call through a fn pointer requests no target. Its
    /// signature is exactly its callee's closed pointer type, compared node by
    /// node, and it relates its arguments and result as a direct call does.
    fn pointer_inputs(
        &mut self,
        original: &s::Original,
        location: OperationSite,
        (signature, callee): (g::TypeRef, u32),
        arguments: g::Range,
        work: &mut Work,
    ) -> Result<(), &'static str> {
        const DIFFERS: &str =
            "source pointer Call signature differs from its callee's pointer type";
        let rows = self.domains.headers.rows();
        let (Some(&s::Type::Function(pointer)), Some(&s::Type::Function(expected))) = (
            rows.types
                .get(original.values[callee as usize].ty.0 as usize),
            rows.types.get(signature.0 as usize),
        ) else {
            return Err(DIFFERS);
        };
        let (actual, declared) = (
            pointer.parameters.slice(&rows.roots)?,
            expected.parameters.slice(&rows.roots)?,
        );
        if pointer.convention != expected.convention
            || pointer.variadic != expected.variadic
            || pointer.unsafe_ != expected.unsafe_
            || actual.len() != declared.len()
        {
            return Err(DIFFERS);
        }
        let results = std::iter::once((&pointer.result, &expected.result));
        for (&actual, &declared) in actual.iter().zip(declared).chain(results) {
            if self.node(original, actual, work)? != self.node(original, declared, work)? {
                return Err(DIFFERS);
            }
        }
        self.call_relations(original, location, signature, arguments, work)
    }
    /// A call relates each argument and its result exactly to its signature.
    fn call_relations(
        &mut self,
        original: &s::Original,
        location: OperationSite,
        signature: g::TypeRef,
        arguments: g::Range,
        work: &mut Work,
    ) -> Result<(), &'static str> {
        let OperationSite { port, site } = location;
        let OperationPort::Value { value, .. } = port else {
            return Err("source Call request has no actual value port");
        };
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
        )
    }
}
impl Builder<'_, '_, '_> {
    /// Version thirteen: an implicit reborrow at a coercion site borrows its
    /// dereferenced place at the expected referent, whose region arguments
    /// are this body's variables. The place's own reference must then be the
    /// actual type of a recorded one-dereference reborrow coercion to exactly
    /// that referent. Every consumer proves that coercion again, relating the
    /// place's referent to the borrowed one under its own variances.
    fn coerced_reborrow(
        &mut self,
        original: &s::Original,
        place: s::Place,
        referent: NodeId,
        work: &mut Work,
    ) -> Result<bool, &'static str> {
        work.charge(1)?;
        let s::Place::Projection(id) = place else {
            return Ok(false);
        };
        let Some(&s::Projection {
            base,
            kind: s::ProjectionKind::Dereference { .. },
            ..
        }) = original.projections.get(id as usize)
        else {
            return Ok(false);
        };
        if !self.domains.bodies {
            return Ok(false);
        }
        let reference = Ports::place_type(original, base, work)?;
        let reference = self.node(original, reference, work)?;
        self.coverage
            .reborrow(original.body, reference, referent, work)
    }
}
/// A borrow whose outermost projection dereferences a reference.
fn reborrowed(original: &s::Original, place: s::Place) -> bool {
    let s::Place::Projection(id) = place else {
        return false;
    };
    original
        .projections
        .get(id as usize)
        .is_some_and(|projection| matches!(projection.kind, s::ProjectionKind::Dereference { .. }))
}

#[cfg(test)]
#[path = "../../../../../../tests/invariants/called_pointer_rows.rs"]
mod tests;
