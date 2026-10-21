//! A selected receiver call shares ordinary callable inference and explicit-place borrowing.
use super::{Checker, Constructor, DefId, Error, NodeId, TermId};
use crate::{
    sema::{
        methods::{Call, receiver, selection::Caller},
        model::{AdjustmentKind, BodyRegionOrigin, Coercion},
    },
    types::{RegionId, infer::Solved},
};

pub(super) struct Pending {
    method: DefId,
    pub(super) callable: TermId,
    dereferences: u32,
    borrow: Option<TermId>,
    actual: TermId,
    expected: TermId,
    unsize: bool,
}
impl Pending {
    pub(super) fn value_types(&self, solved: &Solved) -> [crate::types::TypeId; 3] {
        [self.callable, self.actual, self.expected].map(|term| solved.types[term.0 as usize])
    }
}

impl Checker<'_> {
    pub(super) fn method_arguments(&mut self, node: NodeId) -> Result<(), Error> {
        let site = self
            .report
            .resolution
            .method_calls
            .get(&self.key(node))
            .copied()
            .ok_or_else(|| self.error(node, "missing resolved method call site"))?;
        let term = self.term(site.receiver)?;
        let environment = self.state.method_environment.clone();
        let traits = self.trait_assumptions(node)?;
        let key = self.key(node);
        let selected = receiver::choose(
            self.report,
            self.facts,
            &mut self.state.infer,
            key,
            term,
            site,
            Caller {
                availability: self.availability,
                environment: environment.as_ref(),
                traits: traits.as_deref(),
            },
        )?;
        if selected.from_bound {
            self.facts.methods.bound_uses.insert(key);
        }
        if let Some(subject) = selected.self_term {
            let implementation = self
                .facts
                .associated
                .owner(selected.method)
                .and_then(|id| self.facts.associated.get(id))
                .ok_or_else(|| self.error(node, "missing selected receiver owner"))?
                .self_definition;
            self.state
                .call_types
                .method_owners
                .insert(node, (implementation, subject));
        }
        self.complete_method_arguments(node, selected.method)?;
        let ty = self.facts.definitions[selected.method.0 as usize]
            .ok_or_else(|| self.error(node, "missing selected method type"))?;
        let callable = self.generic_function(node, selected.method, ty)?;
        let signature = self.child(node, callable, 0)?;
        let signature = self.call_signature(node, signature)?;
        let expected = self.child(node, signature, 0)?;
        let (dereferences, borrow, actual) =
            self.adjust_method_receiver(node, site.receiver, term, selected)?;
        let converted = if selected.unsize {
            self.unsize_method_receiver(node, actual)?
        } else {
            actual
        };
        self.shape_equation(node, converted, expected)?;
        self.state.call_types.methods.push((
            node,
            Pending {
                method: selected.method,
                callable,
                dereferences,
                borrow,
                actual,
                expected,
                unsize: selected.unsize,
            },
        ));
        self.state.call_types.active.insert(node, signature);
        self.schedule_call_arguments(node, signature, site.arguments, 1)
    }

    fn unsize_method_receiver(&mut self, node: NodeId, actual: TermId) -> Result<TermId, Error> {
        let Some(reference @ Constructor::Reference { .. }) = self.head(node, actual)? else {
            return Err(self.error(node, "unsized method receiver requires a reference"));
        };
        let array = self.child(node, actual, 0)?;
        let element = self.child(node, array, 0)?;
        let slice = self
            .state
            .infer
            .construct(Constructor::Slice, &[element])
            .map_err(|e| self.error(node, e))?;
        self.state
            .infer
            .construct(reference, &[slice])
            .map_err(|e| self.error(node, e))
    }

    fn adjust_method_receiver(
        &mut self,
        node: NodeId,
        source: NodeId,
        term: TermId,
        selected: receiver::Selected,
    ) -> Result<(u32, Option<TermId>, TermId), Error> {
        let mut actual = term;
        for _ in 0..selected.dereferences {
            actual = self.child(node, actual, 0)?;
        }
        let mut dereferences = selected.dereferences;
        let mut borrow = selected.borrow;
        if borrow.is_none()
            && let Some(Constructor::Reference { mutable, .. }) = self.head(node, actual)?
        {
            dereferences += 1;
            actual = self.child(node, actual, 0)?;
            borrow = Some(mutable);
        }
        if dereferences != 0 {
            // Reborrowing creates a storage place absent from source node facts.
            // Prepare its fields before immutable ownership/loan queries consume
            // that place, including plain generic fields with no projections.
            self.state.call_types.field_receivers.push(actual);
        }
        let borrow = if let Some(mutable) = borrow {
            let region = if self.state.in_function {
                self.body_region(node, BodyRegionOrigin::ExpressionBorrow)?
            } else {
                RegionId::STATIC
            };
            let value = self
                .state
                .infer
                .construct(Constructor::Reference { mutable, region }, &[actual])
                .map_err(|e| self.error(node, e))?;
            if dereferences == 0 {
                self.state.alignment.borrows.push((node, source));
            }
            actual = value;
            Some(value)
        } else {
            None
        };
        Ok((dereferences, borrow, actual))
    }

    pub(super) fn publish_methods(&mut self, solved: &Solved) {
        for index in 0..self.state.call_types.methods.len() {
            let (node, pending) = &self.state.call_types.methods[index];
            let key = self.key(*node);
            let actual = solved.types[pending.actual.0 as usize];
            let expected = solved.types[pending.expected.0 as usize];
            self.facts.methods.calls.insert(
                key,
                Call {
                    method: pending.method,
                    callable: solved.types[pending.callable.0 as usize],
                    dereferences: pending.dereferences,
                    borrow: pending.borrow.map(|term| solved.types[term.0 as usize]),
                    receiver: expected,
                },
            );
            self.facts.retain_coercion(Coercion {
                body: self.state.body,
                source: key,
                actual,
                expected,
                kind: if pending.unsize {
                    AdjustmentKind::ArrayToSlice
                } else {
                    AdjustmentKind::Equality
                },
                reference: None,
            });
        }
    }
}

impl Checker<'_> {
    pub(super) fn retain_language_method(
        &mut self,
        node: NodeId,
        method: DefId,
        callable: TermId,
        receiver: TermId,
    ) {
        self.state.call_types.methods.push((
            node,
            Pending {
                method,
                callable,
                dereferences: 0,
                borrow: None,
                actual: receiver,
                expected: receiver,
                unsize: false,
            },
        ));
    }
}

impl Checker<'_> {
    pub(super) fn retain_language_index(
        &mut self,
        node: NodeId,
        source: NodeId,
        method: DefId,
        callable: TermId,
        subject: TermId,
        receiver: TermId,
    ) {
        self.state.call_types.field_receivers.push(subject);
        self.state.alignment.borrows.push((node, source));
        self.state.call_types.methods.push((
            node,
            Pending {
                method,
                callable,
                dereferences: 0,
                borrow: Some(receiver),
                actual: receiver,
                expected: receiver,
                unsize: false,
            },
        ));
    }
}
