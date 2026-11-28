//! An index operand reaches its indexed place through actual source Deref calls.
use super::{
    Checker, Constructor, Dependency, Error, NodeId, TermId, Work,
    access_paths::{AccessPlanId, OpenAccess, StepRange},
    protocol_control::{OpenKind, OpenPlan},
};
use crate::{sema::compiler_items, syntax::ast::Kind};

impl Checker<'_> {
    pub(super) fn index_plan(&self, node: NodeId) -> Option<AccessPlanId> {
        self.state
            .call_types
            .access
            .as_deref()?
            .indexes
            .get(&node)
            .copied()
    }

    /// A source core that declares both `Deref` and `Index` lets an index
    /// operand take Deref calls on the way to its indexed place.
    pub(super) fn index_operand_steps(&self) -> bool {
        compiler_items::dereferencing::available(self.facts)
            && compiler_items::index_available(self.facts)
    }

    /// One step of the index operand's autoderef. Below its built-in
    /// references the operand is indexed where it is an array or slice or may
    /// implement the index trait. Otherwise a type that may implement `Deref`
    /// contributes one call, and its target resumes here once the projection
    /// worklist has advanced. The calls stay unselected until the mutable-place
    /// choice of the index expression has settled.
    pub(super) fn progress_index(
        &mut self,
        node: NodeId,
        operand: TermId,
    ) -> Result<Option<Dependency>, Error> {
        let Kind::Index { value, index } = self.kind(node) else {
            return Err(self.error(node, "index operand discovery lost its actual source"));
        };
        self.advance_projections(node)?;
        let (subject, dereferences) = self.indexed_referent(node, operand)?;
        if self.index_operand_dereferences(node, subject)? {
            let (step, target) = self.discover_field_target(node, subject, dereferences)?;
            self.extend_index_discovery(node, value, step)?;
            self.state.work.push(Work::Index(node, target));
            return Ok(None);
        }
        if let Some(id) = self.index_plan(node) {
            self.complete_index_discovery(node, id, subject, dereferences)?;
        } else if dereferences != 0 {
            self.facts.autoderefs.insert(self.key(node), dereferences);
        }
        let term = self.indexed_element(node, value, index, subject)?;
        self.schedule_expression_completion(node, term)?;
        Ok(None)
    }

    /// The operand below its built-in references. Rust tries the index trait
    /// on each reference before it dereferences it; an implementation for a
    /// reference type is not selected here, so an operand that may have one
    /// stops instead of indexing its referent.
    pub(super) fn indexed_referent(
        &mut self,
        node: NodeId,
        mut operand: TermId,
    ) -> Result<(TermId, u32), Error> {
        let key = self.key(node);
        let indexing = compiler_items::source_index(self.report, self.facts, key, false)?;
        let assumptions = self.trait_assumptions(node)?;
        for dereferences in 0..1_000_000 {
            if !matches!(
                self.head(node, operand)?,
                Some(Constructor::Reference { .. })
            ) {
                return Ok((operand, dereferences));
            }
            if crate::sema::traits::selection::may_have_subject(
                self.facts,
                &mut self.state.infer,
                crate::types::ItemId(indexing.definition.0),
                operand,
                assumptions.as_deref(),
            )
            .map_err(|e| self.error(node, e))?
            {
                return Err(self.error(
                    node,
                    "an index implementation for a reference type is not selected",
                ));
            }
            operand = self.child(node, operand, 0)?;
        }
        Err(self.error(node, "index operand reference depth exceeded"))
    }

    /// The outer head decides, as for a field or method receiver: the type's
    /// own index implementation wins over its `Deref` implementation, and the
    /// complete index or `Deref` proof remains with the ordinary obligations.
    fn index_operand_dereferences(&mut self, node: NodeId, subject: TermId) -> Result<bool, Error> {
        let Some(head) = self.head(node, subject)? else {
            return Ok(false);
        };
        if matches!(
            head,
            Constructor::Array(_)
                | Constructor::ArrayTerm(_)
                | Constructor::ArrayInfer(_)
                | Constructor::Slice
        ) {
            return Ok(false);
        }
        let key = self.key(node);
        let indexing = compiler_items::source_index(self.report, self.facts, key, false)?;
        let (dereferencing, _) =
            compiler_items::dereferencing::target(self.report, self.facts, key)?;
        let assumptions = self.trait_assumptions(node)?;
        for (owner, steps) in [(indexing.definition, false), (dereferencing, true)] {
            if crate::sema::traits::selection::may_have_subject(
                self.facts,
                &mut self.state.infer,
                crate::types::ItemId(owner.0),
                subject,
                assumptions.as_deref(),
            )
            .map_err(|e| self.error(node, e))?
            {
                return Ok(steps);
            }
        }
        Ok(false)
    }

    fn extend_index_discovery(
        &mut self,
        node: NodeId,
        operand: NodeId,
        index: u32,
    ) -> Result<(), Error> {
        let end = index
            .checked_add(1)
            .ok_or_else(|| self.error(node, "index operand step range overflow"))?;
        let Some(id) = self.index_plan(node) else {
            let slot = self.retain_try(OpenPlan {
                node,
                kind: OpenKind::Index {
                    access: OpenAccess {
                        operand,
                        field: None,
                        receiver: self.term(operand)?,
                        dereferences: 0,
                        steps: StepRange { start: index, end },
                        calls: None,
                    },
                    complete: false,
                },
            })?;
            let id = AccessPlanId::new(slot).map_err(|e| self.error(node, e))?;
            return self.retain_side_plan(node, id, super::deref_coercions::SidePlan::Index);
        };
        match self
            .state
            .call_types
            .protocol_plans
            .get_mut(id.index())
            .filter(|plan| plan.node == node)
            .map(|plan| &mut plan.kind)
        {
            Some(OpenKind::Index {
                access,
                complete: false,
            }) if access.operand == operand && access.steps.end == index => {
                access.steps.end = end;
                Ok(())
            }
            _ => Err(self.error(
                node,
                "index operand discovery changed its source or contiguous steps",
            )),
        }
    }

    fn complete_index_discovery(
        &mut self,
        node: NodeId,
        id: AccessPlanId,
        receiver: TermId,
        dereferences: u32,
    ) -> Result<(), Error> {
        match self
            .state
            .call_types
            .protocol_plans
            .get_mut(id.index())
            .filter(|plan| plan.node == node)
            .map(|plan| &mut plan.kind)
        {
            Some(OpenKind::Index { access, complete }) if !*complete && access.calls.is_none() => {
                access.receiver = receiver;
                access.dereferences = dereferences;
                *complete = true;
                Ok(())
            }
            _ => Err(self.error(node, "index operand discovery completed twice")),
        }
    }

    /// The place a typed index expression indexes: the receiver its operand's
    /// calls reached, or its operand below built-in references.
    pub(super) fn indexed_subject(&mut self, node: NodeId, value: NodeId) -> Result<TermId, Error> {
        let Some(id) = self.index_plan(node) else {
            let operand = self.term(value)?;
            return self.autoderef(node, operand);
        };
        match self
            .state
            .call_types
            .protocol_plans
            .get(id.index())
            .filter(|plan| plan.node == node)
            .map(|plan| plan.kind)
        {
            Some(OpenKind::Index {
                access,
                complete: true,
            }) if access.operand == value => Ok(access.receiver),
            _ => Err(self.error(node, "mutable index lost its completed operand discovery")),
        }
    }

    /// An operand step's `Deref` or `DerefMut` proof is an actual call's. It
    /// is never erased with the built-in index of the same expression.
    pub(super) fn index_step_obligation(&self, index: usize) -> Result<bool, Error> {
        let Some((node, predicate)) = self.state.call_types.obligations.get(index) else {
            return Ok(false);
        };
        let node = *node;
        let Some((item, _)) = predicate.trait_arguments() else {
            return Ok(false);
        };
        if self.index_plan(node).is_none() {
            return Ok(false);
        }
        let key = self.key(node);
        let (shared, _) = compiler_items::dereferencing::target(self.report, self.facts, key)?;
        if item.0 == shared.0 {
            return Ok(true);
        }
        if !self.mutable_language_place(node) {
            return Ok(false);
        }
        let mutable = compiler_items::dereferencing::query(self.report, self.facts, key, true)?;
        Ok(item.0 == mutable.trait_definition.0)
    }
}
