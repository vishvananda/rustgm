//! A selected receiver call shares ordinary callable inference and explicit-place borrowing.
use super::{Checker, Constructor, DefId, Error, NodeId, TermId};
use crate::{
    sema::{
        methods::{Call, receiver, selection::Caller},
        model::{AdjustmentKind, BodyRegionOrigin, Coercion},
    },
    types::infer::Solved,
};

/// One real receiver adjustment also owns its single unsized conversion.
struct Adjusted {
    dereferences: u32,
    borrow: Option<TermId>,
    actual: TermId,
    converted: TermId,
}

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
    pub(super) fn expect_receiver(&mut self, expected: TermId) {
        self.expected = expected;
    }
    pub(super) const fn method(&self) -> DefId {
        self.method
    }
    pub(super) const fn capture_receiver(&self) -> (u32, Option<TermId>) {
        (self.dereferences, self.borrow)
    }
    pub(super) fn value_types(&self, solved: &Solved) -> [crate::types::TypeId; 3] {
        [self.callable, self.actual, self.expected].map(|term| solved.types[term.0 as usize])
    }
}

impl Checker<'_> {
    pub(super) fn method_arguments(
        &mut self,
        node: NodeId,
    ) -> Result<Option<super::Dependency>, Error> {
        let site = self
            .report
            .resolution
            .method_calls
            .get(&self.key(node))
            .copied()
            .ok_or_else(|| self.error(node, "missing resolved method call site"))?;
        let term = self.term(site.receiver)?;
        let environment = self.trait_query_environment(node)?;
        let traits = self.trait_assumptions(node)?;
        let key = self.key(node);
        let progress = receiver::choose(
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
        let selected = match progress {
            receiver::Progress::Ready(selected) => selected,
            receiver::Progress::Prepare(definition) => {
                self.state.work.push(super::Work::MethodArguments(node));
                return Ok(Some(super::Dependency {
                    requirement: super::Requirement::Definition(definition),
                    source: key,
                }));
            }
        };
        if selected.from_bound {
            self.facts.methods.bound_uses.insert(key);
        }
        let adjusted = self.bind_selected_receiver(node, site.receiver, term, selected)?;
        self.facts.methods.demand_library_method(selected.method);
        self.complete_method_arguments(node, selected.method)?;
        let ty = self.facts.definitions[selected.method.0 as usize]
            .ok_or_else(|| self.error(node, "missing selected method type"))?;
        let callable = self.generic_function(node, selected.method, ty)?;
        let signature = self.child(node, callable, 0)?;
        let signature = self.call_signature(node, signature)?;
        let expected = self.child(node, signature, 0)?;
        let Adjusted {
            dereferences,
            borrow,
            actual,
            converted,
        } = if let Some(adjusted) = adjusted {
            adjusted
        } else {
            self.receiver_adjustment(node, site.receiver, term, selected)?
        };
        if borrow.is_some() && self.report.package.uses_source_bindings() {
            self.check_raw_implicit_receiver(node, site.receiver, selected.method)?;
        }
        if let Some(borrow) = borrow
            && matches!(
                self.head(node, borrow)?,
                Some(Constructor::Reference { mutable: true, .. })
            )
        {
            self.mark_mutable_language_place(site.receiver)?;
        }
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
        let target_function = self.selected_target_function(node, callable)?;
        self.retain_active_call(node, signature, target_function)?;
        self.schedule_call_arguments(node, signature, site.arguments, 1)?;
        Ok(None)
    }

    /// This slice supports safe-reference source methods, not the still-required
    /// `dangerous_implicit_autorefs` lint on a synthesized raw-pointer borrow.
    fn check_raw_implicit_receiver(
        &mut self,
        call: NodeId,
        mut receiver: NodeId,
        method: DefId,
    ) -> Result<(), Error> {
        use crate::syntax::ast::{Kind, Unary};
        loop {
            self.state.steps = self.state.steps.saturating_add(1);
            if self.state.steps > 1_000_000 {
                return Err(self.error(call, "raw receiver place work budget exceeded"));
            }
            match self.kind(receiver) {
                Kind::Group(base)
                | Kind::Attributed { value: base, .. }
                | Kind::Field { value: base, .. }
                | Kind::Index { value: base, .. } => receiver = base,
                Kind::Unary {
                    op: Unary::Dereference,
                    operand,
                } => {
                    let term = self.term(operand)?;
                    if matches!(self.head(call, term)?, Some(Constructor::Pointer { .. })) {
                        if !matches!(
                            self.facts.no_implicit_autorefs(self.report, method)?,
                            crate::sema::header_attributes::ContractKnowledge::KnownAbsent
                        ) {
                            return Err(self.error(call,
                                "source marked raw-pointer implicit receiver use is not implemented"));
                        }
                        return Ok(());
                    }
                    receiver = operand;
                }
                // An explicit reference is not an implicit raw-pointer borrow.
                _ => return Ok(()),
            }
        }
    }

    /// Bind structural Self to the exact fresh borrow before generic arguments.
    fn bind_selected_receiver(
        &mut self,
        node: NodeId,
        receiver: NodeId,
        term: TermId,
        selected: receiver::Selected,
    ) -> Result<Option<Adjusted>, Error> {
        let structural = selected.self_term.is_some()
            && self
                .facts
                .associated
                .schema(selected.method)
                .is_some_and(|schema| {
                    !matches!(
                        self.facts.types.kind(schema.self_type),
                        Some(crate::types::Kind::Adt(..))
                    )
                });
        // A structural Self may itself be an autoref. Bind it to the real fresh
        // borrow, never the static region used only in isolated shape probes.
        let adjusted = if structural {
            Some(self.receiver_adjustment(node, receiver, term, selected)?)
        } else {
            None
        };
        if let Some(mut subject) = selected.self_term {
            if let Some(adjusted) = &adjusted {
                subject = adjusted.converted;
                let references = self
                    .facts
                    .methods
                    .receivers
                    .get(&selected.method)
                    .ok_or_else(|| self.error(node, "missing structural receiver shape"))?
                    .1;
                for _ in 0..references {
                    subject = self.child(node, subject, 0)?;
                }
            }
            let implementation = self
                .facts
                .associated
                .member_owner(selected.method)
                .ok_or_else(|| self.error(node, "missing selected receiver owner"))?;
            self.state
                .call_types
                .method_owners
                .insert(node, (implementation, subject));
        }
        Ok(adjusted)
    }

    fn receiver_adjustment(
        &mut self,
        node: NodeId,
        receiver: NodeId,
        term: TermId,
        selected: receiver::Selected,
    ) -> Result<Adjusted, Error> {
        let (dereferences, borrow, actual) =
            self.adjust_method_receiver(node, receiver, term, selected)?;
        let converted = if selected.unsize {
            self.unsize_method_receiver(node, actual)?
        } else {
            actual
        };
        Ok(Adjusted {
            dereferences,
            borrow,
            actual,
            converted,
        })
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
            let region = self.body_region(node, BodyRegionOrigin::ExpressionBorrow)?;
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

    pub(super) fn publish_methods(&mut self, solved: &Solved) -> Result<(), Error> {
        for index in 0..self.state.call_types.comparison_borrows.len() {
            let (node, actual, expected) = self.state.call_types.comparison_borrows[index];
            let key = self.key(node);
            let (actual, expected) = (
                solved.types[actual.0 as usize],
                solved.types[expected.0 as usize],
            );
            self.facts.methods.retain_comparison_borrow(key, actual);
            self.facts.retain_coercion(Coercion {
                body: self.state.body,
                source: key,
                actual,
                expected,
                kind: AdjustmentKind::Equality,
                reference: None,
            });
        }
        for index in 0..self.state.call_types.methods.len() {
            let (node, pending) = &self.state.call_types.methods[index];
            let key = self.key(*node);
            let actual = solved.types[pending.actual.0 as usize];
            let expected = solved.types[pending.expected.0 as usize];
            if self.source_scalar_unary_type(*node, actual)
                || self.source_scalar_binary_solved(*node, actual, solved)?
                || self.source_scalar_comparison_solved(*node, solved)?
            {
                // Its actual source clauses were checked by the ordinary solver;
                // emitting the primitive operation must not call its own body.
                self.state.checks.push(*node);
                continue;
            }
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
        Ok(())
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
    pub(super) fn retain_language_borrow(
        &mut self,
        node: NodeId,
        source: NodeId,
        method: DefId,
        callable: TermId,
        subject: TermId,
        receiver: TermId,
    ) -> usize {
        self.state.call_types.field_receivers.push(subject);
        self.state.alignment.borrows.push((node, source));
        let pending = (
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
        );
        if let Some(index) = self.language_index_method(node) {
            self.state.call_types.methods[index] = pending;
            index
        } else {
            let index = self.state.call_types.methods.len();
            self.state.call_types.methods.push(pending);
            index
        }
    }
}
