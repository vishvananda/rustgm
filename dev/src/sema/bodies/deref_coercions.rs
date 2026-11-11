//! Reference conversions retain actual calls separately from the source operation.
use super::{Checker, Constructor, Error, NodeId, TermId};
use super::{
    access_paths::{AccessPlanId, OpenAccess, StepRange},
    protocol_control::{OpenKind, OpenPlan},
};
use crate::{
    sema::model::{AdjustmentKind, BodyRegionOrigin, ReferenceConversion},
    support::retained,
};

impl Checker<'_> {
    pub(super) fn coercion_plan(&self, node: NodeId) -> Option<AccessPlanId> {
        self.state
            .call_types
            .access
            .as_deref()?
            .conversions
            .get(&node)
            .copied()
    }

    pub(super) fn deref_reference_coercion(
        &mut self,
        node: NodeId,
        actual: TermId,
        expected: TermId,
        converted: TermId,
        contextual: bool,
    ) -> Result<bool, Error> {
        if !crate::sema::compiler_items::dereferencing::available(self.facts) {
            return Ok(false);
        }
        // Prefer the already available built-in conversion. Exploring an
        // unused Deref path must not leave its conditional trait debts behind.
        match self
            .state
            .infer
            .incompatible(&self.facts.types, converted, expected)
        {
            Ok(false) | Err(crate::types::infer::Error::Normalization) => return Ok(false),
            Ok(true) => {}
            Err(error) => return Err(self.error(node, error)),
        }
        let (
            Some(Constructor::Reference { mutable: from, .. }),
            Some(Constructor::Reference { mutable, .. }),
        ) = (self.head(node, actual)?, self.head(node, expected)?)
        else {
            return Ok(false);
        };
        if mutable && !from {
            return Ok(false);
        }
        let target = self.child(node, expected, 0)?;
        if self.head(node, target)?.is_none() {
            return Ok(false);
        }
        let id = if let Some(id) = self.coercion_plan(node) {
            id
        } else {
            let Some(access) = self.discover_reference_calls(node, actual, target)? else {
                return Ok(false);
            };
            if self.propagate_reference(node, expected, contextual)? {
                return Ok(true);
            }
            let slot = self.retain_try(OpenPlan {
                node,
                kind: OpenKind::Coercion { access, mutable },
            })?;
            let id = AccessPlanId::new(slot).map_err(|e| self.error(node, e))?;
            self.retain_reference_calls(node, id)?;
            id
        };
        let access = self.state.call_types.protocol_plans[id.index()]
            .kind
            .access()
            .ok_or_else(|| self.error(node, "reference conversion lost its actual access"))?;
        self.shape_equation(node, access.receiver, target)?;
        let region = self.body_region(node, BodyRegionOrigin::Inference)?;
        let borrow = self
            .state
            .infer
            .construct(
                Constructor::Reference { mutable, region },
                &[access.receiver],
            )
            .map_err(|e| self.error(node, e))?;
        self.state.reference_adjustments.insert(
            self.state.adjustments.len(),
            (
                ReferenceConversion {
                    dereferences: access.dereferences + 1,
                    unsize: false,
                    two_phase: false,
                },
                borrow,
            ),
        );
        self.record_adjustment(node, expected, AdjustmentKind::Reborrow)?;
        Ok(true)
    }

    fn discover_reference_calls(
        &mut self,
        node: NodeId,
        actual: TermId,
        target: TermId,
    ) -> Result<Option<OpenAccess>, Error> {
        let mut source = self.child(node, actual, 0)?;
        let mut dereferences = 1;
        let mut range: Option<StepRange> = None;
        for _ in 0..1_000_000 {
            self.state
                .infer
                .account_work(1)
                .map_err(|e| self.error(node, e))?;
            if !self
                .state
                .infer
                .incompatible(&self.facts.types, source, target)
                .map_err(|e| self.error(node, e))?
            {
                return Ok(range.map(|steps| OpenAccess {
                    operand: node,
                    field: None,
                    receiver: source,
                    dereferences,
                    steps,
                    calls: None,
                }));
            }
            if matches!(
                self.head(node, source)?,
                Some(Constructor::Reference { .. })
            ) {
                source = self.child(node, source, 0)?;
                dereferences += 1;
                continue;
            }
            if self.head(node, source)?.is_none()
                || !self.may_dereference_reference(node, source)?
            {
                return Ok(None);
            }
            let (index, next) = self.discover_field_target(node, source, dereferences)?;
            let end = index
                .checked_add(1)
                .ok_or_else(|| self.error(node, "reference conversion step range overflow"))?;
            if let Some(steps) = &mut range {
                if steps.end != index {
                    return Err(
                        self.error(node, "reference conversion changed its contiguous calls")
                    );
                }
                steps.end = end;
            } else {
                range = Some(StepRange { start: index, end });
            }
            source = next;
            dereferences = 0;
            self.advance_projections(node)?;
        }
        Err(self.error(node, "reference conversion discovery budget exceeded"))
    }

    /// A borrowed constructor retains its own nominal and field inputs even
    /// when the pointee hint names a destination reached by a later Deref call.
    pub(super) fn constructor_deref_destination(
        &mut self,
        node: NodeId,
        actual: TermId,
        expected: TermId,
    ) -> Result<bool, Error> {
        if !crate::sema::compiler_items::dereferencing::available(self.facts) {
            return Ok(false);
        }
        match self
            .state
            .infer
            .incompatible(&self.facts.types, actual, expected)
        {
            Ok(false) | Err(crate::types::infer::Error::Normalization) => Ok(false),
            Ok(true) => self.may_dereference_reference(node, actual),
            Err(e) => Err(self.error(node, e)),
        }
    }

    fn may_dereference_reference(&mut self, node: NodeId, subject: TermId) -> Result<bool, Error> {
        let (owner, _) = crate::sema::compiler_items::dereferencing::target(
            self.report,
            self.facts,
            self.key(node),
        )?;
        let assumptions = self.trait_assumptions(node)?;
        crate::sema::traits::selection::may_have_subject(
            self.facts,
            &mut self.state.infer,
            crate::types::ItemId(owner.0),
            subject,
            assumptions.as_deref(),
        )
        .map_err(|e| self.error(node, e))
    }

    fn retain_reference_calls(&mut self, node: NodeId, id: AccessPlanId) -> Result<(), Error> {
        let rows = self
            .state
            .call_types
            .access
            .as_deref()
            .ok_or_else(|| self.error(node, "reference conversion lost its discovery owner"))?;
        let growth = retained::Growth::map::<NodeId, AccessPlanId>(
            rows.conversions.len(),
            rows.conversions.capacity(),
            1,
        );
        crate::sema::header_attributes::charge_source_retention(
            self.facts,
            growth.work.saturating_add(1),
            growth.bytes,
        )?;
        let source = self.key(node);
        let report = self.report;
        let rows = self.state.call_types.access.as_deref_mut().ok_or_else(|| {
            super::Declarations::error(report, source, "reference conversion owner disappeared")
        })?;
        rows.conversions.try_reserve(1).map_err(|_| {
            super::Declarations::error(report, source, "reference conversion allocation failed")
        })?;
        if growth.bytes != 0 && retained::map(&rows.conversions) > growth.bytes {
            return Err(self.error(node, "reference conversion exceeded its paid storage"));
        }
        rows.conversions.insert(node, id);
        Ok(())
    }

    /// The last actual call supplies the reference ancestry for the normal proof.
    pub(super) fn coercion_reference_view(&self, index: usize) -> Result<Option<TermId>, Error> {
        let adjustment = &self.state.adjustments[index];
        if !self.state.reference_adjustments.contains_key(&index) {
            return Ok(None);
        }
        let Some(id) = self.coercion_plan(adjustment.node) else {
            return Ok(None);
        };
        let access = self.state.call_types.protocol_plans[id.index()]
            .kind
            .access()
            .ok_or_else(|| {
                self.error(adjustment.node, "conversion proof lost its actual access")
            })?;
        self.state
            .call_types
            .access
            .as_deref()
            .and_then(|rows| rows.steps.get(access.steps.end.saturating_sub(1) as usize))
            .and_then(|step| step.output_view)
            .map(Some)
            .ok_or_else(|| {
                self.error(
                    adjustment.node,
                    "conversion proof lost its actual returned reference",
                )
            })
    }
}
