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
            s::ValueKind::Unit | s::ValueKind::Validate(_) => self.unit(original, value.ty, work),
            s::ValueKind::Aggregate { fields } => {
                self.aggregate_value(original, (value.ty, None), fields, work)
            }
            s::ValueKind::VariantAggregate { variant, fields } => {
                self.aggregate_value(original, (value.ty, Some(variant)), fields, work)
            }
            s::ValueKind::IsVariant { place, variant } => {
                self.variant_test(original, (value.ty, place), variant, work)
            }
            s::ValueKind::EnumCast(operand) | s::ValueKind::DirectEnumCast(operand) => {
                self.enum_cast(original, value.ty, operand, work)
            }
            s::ValueKind::Address(_)
            | s::ValueKind::Length(_)
            | s::ValueKind::TextEquals { .. }
            | s::ValueKind::Union { .. }
            | s::ValueKind::Repeat { .. }
            | s::ValueKind::Unsize { .. } => {
                self.extended_value_input(original, value.ty, value.kind, work)
            }
            // Domain validation checked the exact primitive type and canonical bits,
            // an assembly effect's unit type and operand types, and a constant
            // read's declared scalar type.
            // Formation above remains an ordinary unpaid request.
            s::ValueKind::Scalar(_)
            | s::ValueKind::ScalarCast(_)
            | s::ValueKind::PointerCast(_)
            | s::ValueKind::AddressCast {
                kind:
                    s::AddressCast::PointerThin
                    | s::AddressCast::PointerToInteger
                    | s::AddressCast::IntegerToPointer,
                ..
            }
            | s::ValueKind::Unary { .. }
            | s::ValueKind::Binary { .. }
            | s::ValueKind::Text(_)
            | s::ValueKind::Assembly { .. }
            | s::ValueKind::Constant(_)
            | s::ValueKind::AssociatedConstant { .. }
            | s::ValueKind::ConstArgument(_) => Ok(()),
            s::ValueKind::Read(place) => self.read_input(
                original,
                OperationSite { port, site },
                value.ty,
                place,
                work,
            ),
            s::ValueKind::Borrow(place)
            | s::ValueKind::TwoPhaseBorrow(place)
            | s::ValueKind::UniqueBorrow(place) => {
                let mode = match value.kind {
                    s::ValueKind::TwoPhaseBorrow(_) => BorrowMode::TwoPhase,
                    s::ValueKind::UniqueBorrow(_) => BorrowMode::Unique,
                    _ => BorrowMode::Immediate,
                };
                self.borrow_input(
                    original,
                    ports,
                    OperationSite { port, site },
                    value.ty,
                    (place, mode),
                    work,
                )
            }
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
            s::ValueKind::Reify(operand)
            | s::ValueKind::AddressCast {
                kind: s::AddressCast::FunctionPointer,
                operand,
            } => self.reified(original, value.ty, operand, work),
            s::ValueKind::Invoke { .. }
            | s::ValueKind::Call { .. }
            | s::ValueKind::PointerCall { .. } => {
                self.callable_value(original, ports, OperationSite { port, site }, value, work)
            }
        }
    }
    fn callable_value(
        &mut self,
        original: &s::Original,
        ports: &Ports,
        location: OperationSite,
        value: s::Value,
        work: &mut Work,
    ) -> Result<(), &'static str> {
        match value.kind {
            s::ValueKind::Invoke {
                signature,
                arguments,
                ..
            } => self.invoke_inputs(original, location, (value.ty, signature), arguments, work),
            s::ValueKind::Call {
                signature,
                arguments,
                ..
            } => self.call_inputs(original, ports, location, signature, arguments, work),
            s::ValueKind::PointerCall {
                signature,
                callee,
                arguments,
                ..
            } => self.pointer_inputs(original, location, (signature, callee), arguments, work),
            _ => Ok(()),
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
        let Some(&s::Type::Function(signature)) =
            self.domains.headers.rows().types.get(ty.0 as usize)
        else {
            return Err("source reification lost its function item or pointer type");
        };
        let pointer = self.node(original, ty, work)?;
        if let s::ValueKind::Function(application) = original.values[operand as usize].kind {
            let caller = Context {
                owner: original.declaration,
                frame: Some(
                    self.domains.headers.rows().body_inputs
                        [self.domains.headers.original_index(original.body)?]
                    .frame,
                ),
                body: Some(original.body),
            };
            return self.arena.reified(
                self.domains,
                application,
                caller,
                (signature, pointer),
                work,
            );
        }
        if !self.domains.declaration_policies {
            return Err("source reification lost its function item or pointer type");
        }
        let operand_ty = original.values[operand as usize].ty;
        let rows = self.domains.headers.rows();
        let Some(&s::Type::Closure(closure_ref)) = rows.types.get(operand_ty.0 as usize) else {
            return Err("source reification lost its function item or pointer type");
        };
        let closure_type = rows
            .closures()
            .and_then(|closures| closures.types.get(closure_ref.0 as usize))
            .ok_or("source reified closure lost its type row")?;
        let Some(&s::Type::Function(from_sig)) = rows.types.get(closure_type.signature.0 as usize)
        else {
            return Err("source reified closure lost its signature");
        };
        if from_sig.convention != signature.convention
            || from_sig.variadic != signature.variadic
            || (from_sig.unsafe_ && !signature.unsafe_)
        {
            return Err("reified closure pointer differs from its closure signature");
        }
        let operand_node = self.node(original, operand_ty, work)?;
        let Node::Closure {
            signature: from_node,
            environment,
            ..
        } = self.arena.nodes[operand_node.0 as usize]
        else {
            return Err("source reified closure lost its closure node");
        };
        if self.arena.nodes[environment.0 as usize]
            != Node::Scalar(crate::support::crates::Scalar::Unit)
        {
            return Err("capturing closure cannot be reified to a function pointer");
        }
        let (
            Node::Function {
                signature: from_tail,
                ..
            },
            Node::Function {
                signature: to_tail, ..
            },
        ) = (
            self.arena.nodes[from_node.0 as usize],
            self.arena.nodes[pointer.0 as usize],
        )
        else {
            return Err("source reified closure lost its function signature node");
        };
        self.arena
            .same(
                self.domains,
                from_tail,
                to_tail,
                Opening::Relation {
                    body: original.body,
                },
                work,
            )
            .map_err(|_| "reified closure pointer differs from its closure signature")
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
        let expected = Ports::place_type(self.domains.headers, original, place, work)?;
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
        borrow: (s::Place, BorrowMode),
        work: &mut Work,
    ) -> Result<(), &'static str> {
        let OperationSite { port, site } = location;
        let (place, mode) = borrow;
        let two_phase = mode == BorrowMode::TwoPhase;
        let rows = self.domains.headers.rows();
        let s::Type::Reference {
            mutable,
            region,
            referent,
        } = rows.types[ty.0 as usize]
        else {
            return Err("source Borrow has no actual reference result");
        };
        let actual = Ports::place_type(self.domains.headers, original, place, work)?;
        let referent = self.node(original, referent, work)?;
        let allow_temporary = self.domains.declaration_policies;
        let access_ok = match mode {
            BorrowMode::Immediate => {
                !mutable || ports.writable(original, place, allow_temporary, work)?
            }
            BorrowMode::TwoPhase => {
                mutable && ports.writable(original, place, allow_temporary, work)?
            }
            BorrowMode::Unique => mutable && ports.unique(place),
        };
        if self.node(original, actual, work)? != referent
            && !self.coerced_reborrow(original, place, referent, two_phase, work)?
            || !access_ok
        {
            return Err("source Borrow changed its actual referent or writable place");
        }
        if (matches!(place, s::Place::Static(_))
            || self.domains.declaration_policies
                && !mutable
                && matches!(place, s::Place::Temporary(_)))
            && rows.regions[region.0 as usize] == s::Region::Static
        {
            super::uses::region(self.domains, &self.arena, region, Rewrite::Original, work)?;
            return Ok(());
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
            // Version fourteen also admits protocol-plan borrows (`for`, `Deref`)
            // whose region came from the protocol call opening.
            Some(s::RegionAllocation::Inference {
                needs_provenance: false,
            }) if self.domains.declaration_policies
                || self.domains.bodies && reborrowed(original, place) =>
            {
                false
            }
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
            let actual_node = self.node(original, actual, work)?;
            let declared_node = self.node(original, declared, work)?;
            if actual_node != declared_node {
                if !self.domains.declaration_policies {
                    return Err(DIFFERS);
                }
                self.arena
                    .same(
                        self.domains,
                        actual_node,
                        declared_node,
                        Opening::Relation {
                            body: original.body,
                        },
                        work,
                    )
                    .map_err(|_| DIFFERS)?;
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
    fn extended_value_input(
        &mut self,
        original: &s::Original,
        ty: g::TypeRef,
        kind: s::ValueKind,
        work: &mut Work,
    ) -> Result<(), &'static str> {
        match kind {
            s::ValueKind::Address(place) => self.address_input(original, ty, place, work),
            s::ValueKind::Length(place) => self.length_input(original, ty, place, work),
            s::ValueKind::TextEquals { place, text } => {
                self.text_equals_input(original, ty, place, text, work)
            }
            s::ValueKind::Union {
                owner,
                ordinal,
                value: operand,
            } => self.union_value(original, ty, owner, ordinal, operand, work),
            s::ValueKind::Repeat { value: operand, .. } => {
                self.repeat_input(original, ty, operand, work)
            }
            s::ValueKind::Unsize {
                value: operand,
                conversion,
            } => self.unsize_input(original, ty, operand, conversion, work),
            _ => Err("unexpected extended value input"),
        }
    }
    fn address_input(
        &mut self,
        original: &s::Original,
        ty: g::TypeRef,
        place: s::Place,
        work: &mut Work,
    ) -> Result<(), &'static str> {
        let result = self.node(original, ty, work)?;
        let Node::RawPointer { pointee, .. } = self.arena.nodes[result.0 as usize] else {
            return Err("source Address result is not a raw pointer");
        };
        let actual = Ports::place_type(self.domains.headers, original, place, work)?;
        if self.node(original, actual, work)? != pointee {
            return Err("source Address pointee differs from its place type");
        }
        Ok(())
    }
    fn length_input(
        &mut self,
        original: &s::Original,
        ty: g::TypeRef,
        place: s::Place,
        work: &mut Work,
    ) -> Result<(), &'static str> {
        let result = self.node(original, ty, work)?;
        if self.arena.nodes[result.0 as usize]
            != Node::Scalar(crate::support::crates::Scalar::Usize)
        {
            return Err("source Length result is not usize");
        }
        let actual = Ports::place_type(self.domains.headers, original, place, work)?;
        let base = self.node(original, actual, work)?;
        if !matches!(
            self.arena.nodes[base.0 as usize],
            Node::Slice(_) | Node::Array { .. }
        ) {
            return Err("source Length requires a slice or array place");
        }
        Ok(())
    }
    fn text_equals_input(
        &mut self,
        original: &s::Original,
        ty: g::TypeRef,
        place: s::Place,
        text: u32,
        work: &mut Work,
    ) -> Result<(), &'static str> {
        let result = self.node(original, ty, work)?;
        if self.arena.nodes[result.0 as usize] != Node::Scalar(crate::support::crates::Scalar::Bool)
        {
            return Err("source TextEquals result is not bool");
        }
        let actual = Ports::place_type(self.domains.headers, original, place, work)?;
        let base = self.node(original, actual, work)?;
        match self.arena.nodes[base.0 as usize] {
            Node::Str => {
                let bytes = original
                    .texts
                    .as_deref()
                    .and_then(|texts| texts.rows.get(text as usize))
                    .ok_or("source text column differs from its text values")?;
                work.charge(bytes.len().saturating_add(1))?;
                if std::str::from_utf8(bytes).is_err() {
                    return Err("source text literal is not UTF-8");
                }
            }
            Node::Slice(_) | Node::Array { .. } => {}
            _ => return Err("source TextEquals requires a str or byte sequence place"),
        }
        Ok(())
    }
    fn repeat_input(
        &mut self,
        original: &s::Original,
        ty: g::TypeRef,
        operand: u32,
        work: &mut Work,
    ) -> Result<(), &'static str> {
        let result = self.node(original, ty, work)?;
        let Node::Array { element, .. } = self.arena.nodes[result.0 as usize] else {
            return Err("source Repeat result is not an array");
        };
        let actual = self.node(original, original.values[operand as usize].ty, work)?;
        if actual != element {
            return Err("source Repeat operand differs from its array element type");
        }
        Ok(())
    }
    fn unsize_input(
        &mut self,
        original: &s::Original,
        ty: g::TypeRef,
        operand: u32,
        conversion: s::UnsizeConversion,
        work: &mut Work,
    ) -> Result<(), &'static str> {
        let from_node = self.node(original, original.values[operand as usize].ty, work)?;
        let to_node = self.node(original, ty, work)?;
        match conversion {
            s::UnsizeConversion::Slice(array) => {
                self.node(original, array, work)?;
            }
            s::UnsizeConversion::Object { concrete, target } => {
                self.node(original, concrete, work)?;
                self.node(original, target, work)?;
            }
            s::UnsizeConversion::Upcast { source, target }
            | s::UnsizeConversion::Bound { source, target } => {
                self.node(original, source, work)?;
                self.node(original, target, work)?;
            }
            s::UnsizeConversion::Nominal { source, target } => {
                let expected_from = self.node(original, source, work)?;
                let expected_to = self.node(original, target, work)?;
                if from_node != expected_from || to_node != expected_to {
                    return Err("source nominal unsizing changed its operand or result type");
                }
                return Ok(());
            }
        }
        let strip = |mut node: NodeId| {
            if let Node::Pattern { base, .. } = self.arena.nodes[node.0 as usize] {
                node = base;
            }
            self.arena.nodes[node.0 as usize]
        };
        let valid = match (strip(from_node), strip(to_node)) {
            (Node::Reference { mutable: a, .. }, Node::Reference { mutable: b, .. })
            | (Node::Reference { mutable: a, .. }, Node::RawPointer { mutable: b, .. })
            | (Node::RawPointer { mutable: a, .. }, Node::RawPointer { mutable: b, .. }) => a || !b,
            _ => false,
        };
        if !valid {
            return Err("source pointer unsizing requires compatible pointer or reference types");
        }
        Ok(())
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
        two_phase: bool,
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
        let reference = Ports::place_type(self.domains.headers, original, base, work)?;
        let reference = self.node(original, reference, work)?;
        self.coverage
            .reborrow(original.body, reference, referent, two_phase, work)
    }
}
#[derive(Clone, Copy, PartialEq, Eq)]
enum BorrowMode {
    Immediate,
    TwoPhase,
    Unique,
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
