//! Generic reborrowing requires a proved source trait and a fresh actual loan.
use super::{Checker, Constructor, Error, NodeId, TermId};
use crate::{
    sema::{
        model::AdjustmentKind,
        traits::selection::{self, Caller, Selection},
    },
    support::{bindings::LangItem, hash::HashMap, retained::Growth},
    types::{ItemId, infer::predicates::Predicate},
};

pub(super) struct Pending {
    goal: u32,
    mutable: bool,
}
impl Pending {
    pub(in crate::sema::bodies) const fn capture_mutability(&self) -> bool {
        self.mutable
    }
}
impl Checker<'_> {
    pub(super) fn source_value_reborrow(
        &mut self,
        node: NodeId,
        actual: TermId,
        expected: TermId,
    ) -> Result<bool, Error> {
        if !self.state.reference_expected.contains(&node) {
            return Ok(false);
        }
        let (
            Some(Constructor::Adt(a, _) | Constructor::AdtArgs(a, _)),
            Some(Constructor::Adt(b, _) | Constructor::AdtArgs(b, _)),
        ) = (self.head(node, actual)?, self.head(node, expected)?)
        else {
            return Ok(false);
        };
        if !self
            .report
            .package
            .requested_reborrow()
            .map_err(|e| self.error(node, e))?
        {
            return Ok(false);
        }
        let mutable = a == b;
        let role = if mutable {
            LangItem::Reborrow
        } else {
            LangItem::CoerceShared
        };
        let Some(catalog) = self.facts.source_bindings.as_deref() else {
            return Ok(false);
        };
        let Some(declaration) = catalog
            .validated_language(role)
            .map_err(|e| self.error(node, e))?
        else {
            return Ok(false);
        };
        catalog
            .require_provider_report(self.report)
            .map_err(|e| self.error(node, e))?;
        let assumptions = self.trait_assumptions(node)?.unwrap_or_default();
        let environment = self.trait_query_environment(node)?;
        let subject = if mutable { expected } else { actual };
        let item = ItemId(declaration.definition.0);
        if !selection::may_have_subject(
            self.facts,
            &mut self.state.infer,
            item,
            subject,
            Some(&assumptions),
        )
        .map_err(|e| self.error(node, e))?
        {
            return Ok(false);
        }
        if self.propagate_reference(node, expected, true)? {
            return Ok(true);
        }
        let arguments = if mutable {
            Vec::new()
        } else {
            vec![crate::types::infer::arguments::Argument::Type(expected)]
        };
        let goal = Predicate::trait_application(subject, item, arguments);
        let first_region = self.trait_region_start(node)?;
        let selected = selection::select_available(
            self.facts,
            &mut self.state.infer,
            &goal,
            Caller {
                assumptions: &assumptions,
                environment: environment.as_ref(),
                first_region,
                retain_sized: false,
                nesting: crate::sema::traits::selection::Nesting::ROOT,
                placeholders: None,
            },
            self.availability,
        )
        .map_err(|e| self.error(node, e))?;
        let evidence = match selected {
            Selection::Proven(evidence) => evidence,
            Selection::Absent | Selection::Deferred => return Ok(false),
            Selection::Refuted(refutation) => {
                refutation.reject().map_err(|e| self.error(node, e))?;
                return Ok(false);
            }
        };
        self.retain_trait_evidence(node, evidence)?;
        if mutable {
            self.callable_shape(node, actual, expected)?;
        }
        self.retain_value_reborrow(node, goal, mutable)?;
        self.record_adjustment(
            node,
            expected,
            if mutable {
                AdjustmentKind::GenericReborrow
            } else {
                AdjustmentKind::CoerceShared
            },
        )?;
        Ok(true)
    }
    fn retain_value_reborrow(
        &mut self,
        node: NodeId,
        goal: Predicate,
        mutable: bool,
    ) -> Result<(), Error> {
        let index = self.state.call_types.obligations.len();
        let goal_id = u32::try_from(index)
            .map_err(|_| self.error(node, "value reborrow goal inventory exhausted"))?;
        let mut growth = Growth::vector::<(NodeId, Predicate)>(
            index,
            self.state.call_types.obligations.capacity(),
            1,
        );
        let pending = self.state.value_reborrows.as_deref();
        let (length, capacity) = pending.map_or((0, 0), |p| (p.len(), p.capacity()));
        growth.include(Growth::map::<usize, Pending>(length, capacity, 1));
        self.facts
            .charge_caller_inputs(
                growth.work.saturating_add(1),
                growth.bytes.saturating_add(if pending.is_none() {
                    size_of::<HashMap<usize, Pending>>()
                } else {
                    0
                }),
            )
            .map_err(|e| self.error(node, e))?;
        self.state
            .call_types
            .obligations
            .try_reserve(1)
            .map_err(|_| self.error(node, "value reborrow goal allocation failed"))?;
        self.state
            .value_reborrows
            .get_or_insert_default()
            .try_reserve(1)
            .map_err(|_| self.error(node, "value reborrow continuation allocation failed"))?;
        self.state.call_types.obligations.push((node, goal));
        self.state
            .value_reborrows
            .as_deref_mut()
            .ok_or_else(|| {
                crate::sema::model::Declarations::error(
                    self.report,
                    self.state.body,
                    "value reborrow continuation disappeared",
                )
            })?
            .insert(
                self.state.adjustments.len(),
                Pending {
                    goal: goal_id,
                    mutable,
                },
            );
        Ok(())
    }
    pub(super) fn publish_value_reborrow(
        &mut self,
        index: usize,
        solved: &crate::types::infer::Solved,
        predicates: &[crate::types::parameters::predicates::Predicate],
        evidence: &crate::support::hash::HashMap<usize, crate::sema::traits::selection::Source>,
    ) -> Result<Option<crate::types::TypeId>, Error> {
        let Some(pending) = self
            .state
            .value_reborrows
            .as_deref()
            .and_then(|p| p.get(&index))
        else {
            return Ok(None);
        };
        let adjustment = &self.state.adjustments[index];
        let node = adjustment.node;
        let mutable = pending.mutable;
        let goal = pending.goal as usize;
        let source = solved.types[adjustment.actual.0 as usize];
        let target = solved.types[adjustment.expected.0 as usize];
        let role = if mutable {
            LangItem::Reborrow
        } else {
            LangItem::CoerceShared
        };
        let declaration = self
            .facts
            .source_bindings
            .as_deref()
            .ok_or_else(|| self.error(node, "value reborrow catalog disappeared"))?
            .validated_language(role)
            .map_err(|e| self.error(node, e))?
            .ok_or_else(|| self.error(node, "value reborrow role disappeared"))?;
        let predicate = predicates
            .get(goal)
            .ok_or_else(|| self.error(node, "value reborrow goal was not solved"))?;
        let (application, _) = predicate
            .bound
            .trait_parts()
            .ok_or_else(|| self.error(node, "value reborrow lost its actual trait goal"))?;
        let app = self
            .facts
            .types
            .trait_application(application)
            .ok_or_else(|| self.error(node, "value reborrow application disappeared"))?;
        if app.item != ItemId(declaration.definition.0)
            || predicate.ty != if mutable { target } else { source }
            || !evidence.contains_key(&goal)
            || self
                .state
                .call_types
                .obligations
                .get(goal)
                .is_none_or(|(site, _)| *site != node)
        {
            return Err(self.error(node, "value reborrow changed its actual source/goal/role"));
        }
        if self
            .facts
            .types
            .arguments(app.arguments)
            .is_none_or(|mut args| {
                if mutable {
                    args.next().is_some()
                } else {
                    args.next() != Some(crate::types::arguments::Argument::Type(target))
                        || args.next().is_some()
                }
            })
        {
            return Err(self.error(
                node,
                "value reborrow changed its complete target application",
            ));
        }
        self.prepare_storage(node, target)?;
        let field = if mutable {
            None
        } else {
            self.shared_value_regions(node, source, target)?
        };
        crate::sema::reborrow_adjustments::retain(
            self.facts,
            self.key(node),
            crate::sema::reborrow_adjustments::Adjustment {
                body: self.state.body,
                source,
                target,
                borrow: target,
                mutable,
                declaration: declaration.definition,
                field,
            },
        )
        .map_err(|e| self.error(node, e))?;
        Ok((!mutable).then_some(target))
    }
}
