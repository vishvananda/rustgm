//! Field discovery retains actual projection inputs until mutable-place choice settles.
use super::{Checker, Error, NodeId, TermId};
use crate::{
    support::{hash::HashSet, retained},
    types::infer::predicates::Predicate,
};
use std::num::NonZeroU32;

/// An unfinished field plan's location in this body's existing open-plan owner.
/// This is never a checked protocol-plan identity or permission to call a method.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) struct FieldPlanId(NonZeroU32);
impl FieldPlanId {
    pub(super) fn new(index: usize) -> Result<Self, &'static str> {
        let index = u32::try_from(index)
            .ok()
            .and_then(|index| index.checked_add(1))
            .and_then(NonZeroU32::new)
            .ok_or("field open-plan inventory overflow")?;
        Ok(Self(index))
    }
    pub(super) const fn index(self) -> usize {
        (self.0.get() - 1) as usize
    }
}

#[derive(Clone, Copy)]
pub(super) struct StepRange {
    pub start: u32,
    pub end: u32,
}
impl StepRange {
    pub(super) fn len(self) -> Result<usize, &'static str> {
        self.end
            .checked_sub(self.start)
            .map(|length| length as usize)
            .ok_or("field discovery changed its actual step range")
    }
}

#[derive(Clone, Copy)]
pub(super) struct CallRange {
    pub method: u32,
    pub call: u32,
}

#[derive(Clone, Copy)]
pub(super) struct OpenField {
    pub operand: NodeId,
    pub field: Option<crate::sema::members::Selection>,
    pub receiver: TermId,
    pub dereferences: u32,
    pub steps: StepRange,
    pub calls: Option<CallRange>,
}

#[derive(Clone, Copy)]
pub(super) struct OpenStep {
    pub node: NodeId,
    pub subject: TermId,
    pub target: TermId,
    pub output_view: Option<TermId>,
    pub dereferences: u32,
}

const _: [(); 24] = [(); size_of::<OpenStep>()];

/// A prepared call's actual first receiver, never its invocation-time target.
#[derive(Clone, Copy)]
pub(super) struct CaptureReceiver {
    pub operand: NodeId,
    pub dereferences: u32,
    pub borrow: TermId,
}
const _: [(); 12] = [(); size_of::<CaptureReceiver>()];

#[derive(Default)]
pub(super) struct Pending {
    pub steps: Vec<OpenStep>,
    /// Current inference roots, keyed by the actual Field source. No type is erased.
    visited: HashSet<(NodeId, TermId)>,
    /// The existing protocol-plan inventory is visited once, including resumes.
    pub next_plan: usize,
}

impl Checker<'_> {
    /// Mutable-place discovery has settled before this existing body finish port.
    /// Every actual call and constraint enters the one inference snapshot below.
    pub(super) fn finish_field_calls(&mut self) -> Result<(), Error> {
        let Some(rows) = self.state.call_types.access.as_deref() else {
            return Ok(());
        };
        let mut cursor = rows.next_plan;
        while cursor < self.state.call_types.protocol_plans.len() {
            let open = self.state.call_types.protocol_plans[cursor];
            if let super::protocol_control::OpenKind::Field(field) = open.kind {
                if field.calls.is_some() || field.field.is_none() || field.steps.len().is_err() {
                    return Err(
                        self.error(open.node, "field call preparation changed its actual stage")
                    );
                }
                let count = field.steps.len().map_err(|e| self.error(open.node, e))?;
                if count == 0 {
                    return Err(self.error(open.node, "field call plan has no actual steps"));
                }
                let method = u32::try_from(self.state.call_types.methods.len())
                    .map_err(|_| self.error(open.node, "field method inventory overflow"))?;
                let call = u32::try_from(self.state.calls.len())
                    .map_err(|_| self.error(open.node, "field call inventory overflow"))?;
                self.reserve_field_calls(open.node, count)?;
                let mutable = self.mutable_language_place(open.node);
                for index in 0..count {
                    self.prepare_field_step(
                        open.node,
                        field,
                        index,
                        CallRange { method, call },
                        mutable,
                    )?;
                }
                self.state
                    .alignment
                    .borrows
                    .push((open.node, field.operand));
                let super::protocol_control::OpenKind::Field(actual) =
                    &mut self.state.call_types.protocol_plans[cursor].kind
                else {
                    return Err(self.error(open.node, "field call plan changed during preparation"));
                };
                actual.calls = Some(CallRange { method, call });
            }
            cursor += 1;
            self.state
                .call_types
                .access
                .as_deref_mut()
                .ok_or_else(|| {
                    super::Declarations::error(
                        self.report,
                        self.state.body,
                        "field preparation lost its actual staging owner",
                    )
                })?
                .next_plan = cursor;
        }
        Ok(())
    }

    pub(super) fn field_capture_receiver(
        &self,
        node: NodeId,
    ) -> Result<Option<CaptureReceiver>, Error> {
        let Some(id) = self.field_plan(node) else {
            return Ok(None);
        };
        let open = self
            .state
            .call_types
            .protocol_plans
            .get(id.index())
            .copied()
            .filter(|open| open.node == node)
            .ok_or_else(|| self.error(node, "field capture belongs to another actual plan"))?;
        let super::protocol_control::OpenKind::Field(field) = open.kind else {
            return Err(self.error(node, "field capture changed its actual plan kind"));
        };
        if !matches!(self.kind(node), crate::syntax::ast::Kind::Field { value, .. } if value == field.operand)
            || field.field.is_none()
            || field.steps.len().map_err(|e| self.error(node, e))? == 0
        {
            return Err(self.error(node, "field capture lost its actual source or discovery"));
        }
        // Reuse the real contiguous method/call range authentication, retaining
        // the public callable and opened signature for normal final publication.
        self.protocol_open_call(open, 0)
            .map_err(|e| self.error(node, e))?;
        let range = field
            .calls
            .ok_or_else(|| self.error(node, "field capture call is unfinished"))?;
        let step = self
            .state
            .call_types
            .access
            .as_deref()
            .and_then(|rows| rows.steps.get(field.steps.start as usize))
            .filter(|step| step.node == node && step.output_view.is_some())
            .ok_or_else(|| self.error(node, "field capture lost its actual first step"))?;
        let (_, pending) = &self.state.call_types.methods[range.method as usize];
        let (dereferences, borrow) = pending.capture_receiver();
        if dereferences != step.dereferences {
            return Err(self.error(node, "field capture changed its actual receiver prefix"));
        }
        Ok(Some(CaptureReceiver {
            operand: field.operand,
            dereferences,
            borrow: borrow.ok_or_else(|| self.error(node, "field capture has no actual borrow"))?,
        }))
    }

    fn prepare_field_step(
        &mut self,
        node: NodeId,
        field: OpenField,
        index: usize,
        range: CallRange,
        mutable: bool,
    ) -> Result<(), Error> {
        let ordinal = field.steps.start as usize + index;
        let step = self
            .state
            .call_types
            .access
            .as_deref()
            .and_then(|rows| rows.steps.get(ordinal))
            .copied()
            .filter(|step| step.node == node && step.output_view.is_none())
            .ok_or_else(|| {
                self.error(
                    node,
                    "field discovery range changed its actual source or stage",
                )
            })?;
        let selected = self.dereference_call(node, step.subject, mutable)?;
        self.equate(node, selected.target, step.target)?;
        let result = self.child(node, selected.signature, 1)?;
        let Some(super::Constructor::Reference {
            mutable: output_mutable,
            region,
        }) = self.head(node, result)?
        else {
            return Err(self.error(node, "field call lost its actual reference result"));
        };
        let view = self
            .state
            .infer
            .construct(
                super::Constructor::Reference {
                    mutable: output_mutable,
                    region,
                },
                &[step.target],
            )
            .map_err(|e| self.error(node, e))?;
        self.shape_equation(node, result, view)?;
        if index == 0 {
            self.check_raw_projection_borrow(node, field.operand, selected.method, false)?;
        }
        if self.state.call_types.methods.len() != range.method as usize + index
            || self.state.calls.len() != range.call as usize + index
        {
            return Err(self.error(node, "field call opening changed its retained ranges"));
        }
        self.state.call_types.methods.push((
            node,
            super::dot_calls::Pending::borrowed(
                selected.method,
                selected.callable,
                step.dereferences,
                selected.borrow,
                selected.receiver,
            ),
        ));
        self.state.calls.push((node, selected.signature));
        self.state.call_types.field_receivers.push(step.subject);
        let source = self.key(node);
        self.state
            .call_types
            .access
            .as_deref_mut()
            .and_then(|rows| rows.steps.get_mut(ordinal))
            .ok_or_else(|| {
                super::Declarations::error(
                    self.report,
                    source,
                    "field step disappeared during call preparation",
                )
            })?
            .output_view = Some(view);
        Ok(())
    }

    pub(super) fn publish_field_outputs(
        &mut self,
        solved: &crate::types::infer::Solved,
    ) -> Result<(), Error> {
        let Some(rows) = self.state.call_types.access.as_deref() else {
            return Ok(());
        };
        let growth = retained::Growth::vector::<crate::sema::model::Coercion>(
            self.facts.coercions.len(),
            self.facts.coercions.capacity(),
            rows.steps.len(),
        );
        crate::sema::header_attributes::charge_source_retention(
            self.facts,
            rows.steps.len().saturating_add(growth.work),
            growth.bytes,
        )?;
        self.facts
            .coercions
            .try_reserve(rows.steps.len())
            .map_err(|_| {
                self.error(
                    self.state.body.node,
                    "field output coercion allocation failed",
                )
            })?;
        for ordinal in 0..self.state.call_types.protocol_plans.len() {
            let open = self.state.call_types.protocol_plans[ordinal];
            let super::protocol_control::OpenKind::Field(field) = open.kind else {
                continue;
            };
            let calls = field
                .calls
                .ok_or_else(|| self.error(open.node, "field output has no actual call range"))?;
            for index in 0..field.steps.len().map_err(|e| self.error(open.node, e))? {
                let step = self
                    .state
                    .call_types
                    .access
                    .as_deref()
                    .and_then(|rows| rows.steps.get(field.steps.start as usize + index))
                    .ok_or_else(|| self.error(open.node, "field output lost its actual step"))?;
                let view = step
                    .output_view
                    .ok_or_else(|| self.error(open.node, "field output view is unfinished"))?;
                let signature = self
                    .state
                    .calls
                    .get(calls.call as usize + index)
                    .filter(|(node, _)| *node == open.node)
                    .ok_or_else(|| {
                        self.error(open.node, "field output changed its actual opened call")
                    })?
                    .1;
                let Some(super::Type::Function { result: actual, .. }) =
                    self.facts.types.kind(solved.types[signature.0 as usize])
                else {
                    return Err(
                        self.error(open.node, "field output lost its actual call signature")
                    );
                };
                self.facts.retain_coercion(crate::sema::model::Coercion {
                    body: self.state.body,
                    source: self.key(open.node),
                    actual,
                    expected: solved.types[view.0 as usize],
                    kind: crate::sema::model::AdjustmentKind::Equality,
                    reference: None,
                });
            }
        }
        Ok(())
    }

    fn reserve_field_calls(&mut self, node: NodeId, count: usize) -> Result<(), Error> {
        let methods = &self.state.call_types.methods;
        let mut growth = retained::Growth::vector::<(NodeId, super::dot_calls::Pending)>(
            methods.len(),
            methods.capacity(),
            count,
        );
        growth.include(retained::Growth::vector::<(NodeId, TermId)>(
            self.state.calls.len(),
            self.state.calls.capacity(),
            count,
        ));
        growth.include(retained::Growth::vector::<TermId>(
            self.state.call_types.field_receivers.len(),
            self.state.call_types.field_receivers.capacity(),
            count,
        ));
        growth.include(retained::Growth::vector::<(NodeId, NodeId)>(
            self.state.alignment.borrows.len(),
            self.state.alignment.borrows.capacity(),
            1,
        ));
        crate::sema::header_attributes::charge_source_retention(
            self.facts,
            count.saturating_add(growth.work),
            growth.bytes,
        )?;
        let report = self.report;
        let source = self.key(node);
        let error =
            |_| super::Declarations::error(report, source, "field call staging allocation failed");
        self.state
            .call_types
            .methods
            .try_reserve(count)
            .map_err(error)?;
        self.state.calls.try_reserve(count).map_err(error)?;
        self.state
            .call_types
            .field_receivers
            .try_reserve(count)
            .map_err(error)?;
        self.state.alignment.borrows.try_reserve(1).map_err(error)?;
        Ok(())
    }
    pub(super) fn progress_field(
        &mut self,
        node: NodeId,
        original: super::aggregates::Field,
    ) -> Result<Option<super::Dependency>, Error> {
        self.advance_projections(node)?;
        let (receiver, dereferences) = self
            .state
            .infer
            .reference_target(&self.facts.types, original.receiver)
            .map_err(|e| self.error(node, e))?;
        let crate::syntax::ast::Kind::Field { value, .. } = self.kind(node) else {
            return Err(self.error(node, "field receiver read lost its actual source"));
        };
        if dereferences != 0 {
            self.check_union_receiver(node, value)?;
        }
        let field = super::aggregates::Field {
            receiver,
            name: original.name,
        };
        let kind = self.head(node, receiver)?.ok_or_else(|| {
            self.error(node, "field access requires a known tuple or struct type")
        })?;
        if let Some(item) = self.field_dependency(kind) {
            self.state.work.push(super::Work::Field(node, original));
            return Ok(Some(super::Dependency {
                requirement: super::Requirement::Nominal(item),
                source: self.key(node),
            }));
        }
        if self.present_field(node, field, kind)?
            || !crate::sema::compiler_items::dereferencing::available(self.facts)
        {
            let term = self.complete_field(node, field)?;
            if let Some(plan) = self.field_plan(node) {
                self.complete_field_discovery(node, plan, receiver, dereferences)?;
            } else if dereferences != 0 {
                self.facts.autoderefs.insert(self.key(node), dereferences);
            }
            self.schedule_expression_completion(node, term)?;
            return Ok(None);
        }
        // Discovery establishes that this is a call-time receiver borrow, not
        // a direct field store. Check its real lexical unsafe context now;
        // final method selection happens after mutable-place discovery settles.
        if dereferences == 0 && self.field_plan(node).is_none() {
            self.check_union_receiver(node, value)?;
        }
        let (index, target) = self.discover_field_target(node, receiver, dereferences)?;
        self.extend_field_discovery(node, index)?;
        // Resume the same target after the existing projection/declaration worklist.
        // No earlier receiver step or parameter expression is revisited.
        self.state.work.push(super::Work::Field(
            node,
            super::aggregates::Field {
                receiver: target,
                name: original.name,
            },
        ));
        Ok(None)
    }

    fn extend_field_discovery(&mut self, node: NodeId, index: u32) -> Result<(), Error> {
        let crate::syntax::ast::Kind::Field { value: operand, .. } = self.kind(node) else {
            return Err(self.error(node, "field discovery lost its actual source operation"));
        };
        let end = index
            .checked_add(1)
            .ok_or_else(|| self.error(node, "field discovery range overflow"))?;
        if let Some(id) = self.field_plan(node) {
            let source = self.key(node);
            let report = self.report;
            let plan = self
                .state
                .call_types
                .protocol_plans
                .get_mut(id.index())
                .filter(|plan| plan.node == node)
                .ok_or_else(|| {
                    super::Declarations::error(
                        report,
                        source,
                        "field discovery belongs to another open-plan owner",
                    )
                })?;
            let super::protocol_control::OpenKind::Field(field) = &mut plan.kind else {
                return Err(self.error(node, "field discovery changed its actual open-plan kind"));
            };
            if field.operand != operand || field.steps.end != index || field.field.is_some() {
                return Err(self.error(
                    node,
                    "field discovery changed its source or contiguous range",
                ));
            }
            field.steps.end = end;
        } else {
            let plan = self.retain_try(super::protocol_control::OpenPlan {
                node,
                kind: super::protocol_control::OpenKind::Field(OpenField {
                    operand,
                    field: None,
                    receiver: self.term(operand)?,
                    dereferences: 0,
                    steps: StepRange { start: index, end },
                    calls: None,
                }),
            })?;
            let id = FieldPlanId::new(plan).map_err(|e| self.error(node, e))?;
            self.retain_field_plan(node, id)?;
        }
        Ok(())
    }

    fn complete_field_discovery(
        &mut self,
        node: NodeId,
        id: FieldPlanId,
        receiver: TermId,
        dereferences: u32,
    ) -> Result<(), Error> {
        let Some(crate::sema::model::Operation::Field {
            field: selected, ..
        }) = self
            .facts
            .node(self.key(node))
            .and_then(|fact| fact.operation)
        else {
            return Err(self.error(node, "field discovery lacks its actual selected member"));
        };
        let source = self.key(node);
        let report = self.report;
        let plan = self
            .state
            .call_types
            .protocol_plans
            .get_mut(id.index())
            .filter(|plan| plan.node == node)
            .ok_or_else(|| {
                super::Declarations::error(
                    report,
                    source,
                    "field completion belongs to another open-plan owner",
                )
            })?;
        let super::protocol_control::OpenKind::Field(field) = &mut plan.kind else {
            return Err(super::Declarations::error(
                report,
                source,
                "field completion changed its actual open-plan kind",
            ));
        };
        if field.field.is_some() || field.calls.is_some() {
            return Err(super::Declarations::error(
                report,
                source,
                "field discovery completed twice",
            ));
        }
        field.field = Some(selected);
        field.receiver = receiver;
        field.dereferences = dereferences;
        self.state.call_types.field_receivers.push(receiver);
        Ok(())
    }
    /// Discover only the actual associated result. Naming the shared method here
    /// would incorrectly retain its feature/const debts when the final use is mutable.
    pub(super) fn discover_field_target(
        &mut self,
        node: NodeId,
        subject: TermId,
        dereferences: u32,
    ) -> Result<(u32, TermId), Error> {
        self.state.steps = self.state.steps.saturating_add(1);
        if self.state.steps > 1_000_000 {
            return Err(self.error(node, "field discovery work budget exceeded"));
        }
        let (owner, item) = crate::sema::compiler_items::dereferencing::target(
            self.report,
            self.facts,
            self.key(node),
        )?;
        let target = self
            .state
            .infer
            .projected(&self.facts.types, item, subject, &[], &[])
            .map_err(|e| self.error(node, e))?;
        self.state.call_types.obligations.push((
            node,
            Predicate::trait_application(subject, crate::types::ItemId(owner.0), Vec::new()),
        ));
        self.defer_normalization(node, crate::types::lifetime_proof::ScopeId::ROOT, target)?;
        let index = self.retain_field_step(
            node,
            OpenStep {
                node,
                subject,
                target,
                output_view: None,
                dereferences,
            },
        )?;
        Ok((index, target))
    }

    fn retain_field_step(&mut self, node: NodeId, step: OpenStep) -> Result<u32, Error> {
        let subject = self
            .state
            .infer
            .root(step.subject)
            .map_err(|e| self.error(node, e))?;
        let current = self.state.call_types.access.as_deref();
        if current.is_some_and(|rows| rows.visited.contains(&(node, subject))) {
            return Err(self.error(node, "field Deref/Target chain repeats its actual subject"));
        }
        let (length, capacity) =
            current.map_or((0, 0), |rows| (rows.steps.len(), rows.steps.capacity()));
        let index = u32::try_from(length)
            .map_err(|_| self.error(node, "field discovery inventory overflow"))?;
        let growth = retained::Growth::vector::<OpenStep>(length, capacity, 1);
        let map_growth = current.map_or_else(
            || retained::Growth::map::<(NodeId, TermId), ()>(0, 0, 1),
            |rows| {
                retained::Growth::map::<(NodeId, TermId), ()>(
                    rows.visited.len(),
                    rows.visited.capacity(),
                    1,
                )
            },
        );
        let mut visit_growth = map_growth;
        visit_growth.include(growth);
        let bytes = visit_growth.bytes.saturating_add(if current.is_none() {
            size_of::<Pending>() + size_of::<Option<Box<Pending>>>()
        } else {
            0
        });
        crate::sema::header_attributes::charge_source_retention(
            self.facts,
            1usize.saturating_add(visit_growth.work),
            bytes,
        )?;
        let source = self.key(node);
        let report = self.report;
        let rows = self
            .state
            .call_types
            .access
            .get_or_insert_with(Default::default);
        rows.steps.try_reserve(1).map_err(|_| {
            super::Declarations::error(report, source, "field discovery allocation failed")
        })?;
        rows.visited.try_reserve(1).map_err(|_| {
            super::Declarations::error(report, source, "field discovery visit allocation failed")
        })?;
        if growth.bytes != 0 && retained::vector(&rows.steps) > growth.bytes {
            return Err(self.error(node, "field discovery exceeded its paid storage"));
        }
        if map_growth.bytes != 0
            && rows
                .visited
                .capacity()
                .saturating_mul(size_of::<(NodeId, TermId)>())
                > map_growth.bytes
        {
            return Err(self.error(node, "field discovery visits exceeded their paid storage"));
        }
        rows.visited.insert((node, subject));
        rows.steps.push(step);
        Ok(index)
    }
}
