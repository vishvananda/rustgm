//! Source sizing fallbacks retain the actual trait goal and ordinary formation duties.
use super::{
    Action, Decision, Declarations, Frame, Goal, Node, Predicate, Search, Source, inference,
};
use crate::{
    abi::metadata::Metadata,
    resolve::model::DefId,
    support::bindings::LangItem,
    types::{Kind, aggregate::MetadataPreparation},
};
use std::ops::ControlFlow;

#[cfg(test)]
#[path = "../../../../../tests/invariants/placeholder_sizing.rs"]
mod placeholder_tests;

/// rustc 1.98 negates one impl's predicates under the other impl's
/// `param_env`, after plugging each unified header variable with a
/// placeholder. A negative coherence query decides a rigid parameter's `Sized`
/// and `MetaSized` only from that other header's own premises, never from the
/// declaration of the parameter `freeze` made the representative. A `Sized`
/// decision stays an ordinary Sized debt, and each `MetaSized` node stays a
/// debt that the `MetaSized` premises must pay. Every other query has none.
/// The builtin proof's shape walk and size queries read the same premises.
pub(super) struct NegativeSizing {
    premises: crate::types::parameters::shape::Placeholders,
    debts: Vec<usize>,
}
impl NegativeSizing {
    /// The premises are one side's handle, ordered once by
    /// [`Placeholders::new`] and shared by every condition of that pair's
    /// side, so assuming them clones the handle and does no work.
    ///
    /// [`Placeholders::new`]: crate::types::parameters::shape::Placeholders::new
    pub(super) fn assume(premises: &crate::types::parameters::shape::Placeholders) -> Self {
        Self {
            premises: premises.clone(),
            debts: Vec::new(),
        }
    }
    /// Whether the premises give a rigid parameter `kind`, and the lookup's work.
    fn parameter(
        &self,
        kind: LangItem,
        ty: crate::types::TypeId,
    ) -> Result<(bool, usize), &'static str> {
        match kind {
            LangItem::Sized => Ok(self.premises.sized(ty)),
            LangItem::MetaSized => Ok(self.premises.meta_sized(ty)),
            LangItem::PointeeSized => Ok((true, 1)),
            _ => Err("invalid negative coherence sizing kind"),
        }
    }
    /// The premises the shape walk and size queries assume.
    pub(super) fn placeholders(&self) -> &crate::types::parameters::shape::Placeholders {
        &self.premises
    }
    fn record(&mut self, selected: usize) -> Result<(), &'static str> {
        if self.debts.last().is_some_and(|&last| last >= selected) {
            return Err("negative coherence MetaSized debts lost their order");
        }
        self.debts
            .try_reserve(1)
            .map_err(|_| "negative coherence MetaSized debts allocation failed")?;
        self.debts.push(selected);
        Ok(())
    }
    /// Node indices are allocated in increasing order and never reused.
    pub(super) fn debt(&self, index: usize) -> bool {
        self.debts.binary_search(&index).is_ok()
    }
}

impl Search<'_> {
    pub(super) fn source_sizing_kind(
        facts: &Declarations,
        goal: &Goal,
    ) -> Result<Option<LangItem>, String> {
        let Some(catalog) = facts.source_bindings.as_deref() else {
            return Ok(None);
        };
        let Some((item, arguments)) = goal.trait_arguments() else {
            return Ok(None);
        };
        let kind = catalog.sizing_kind(DefId(item.0))?;
        if let Some(kind) = kind {
            let declaration = catalog
                .validated_role(kind)?
                .ok_or("source sizing goal lost its actual validated declaration")?;
            if matches!(
                declaration.origin,
                crate::resolve::body_origin::DeclarationOrigin::Imported(_)
            ) {
                facts.associated.require_imported_role(declaration)?;
            }
        }
        if kind.is_some()
            && goal.constness() != Some(crate::support::constness::Constness::Ordinary)
        {
            return Err(
                "source sizing applications require their actual ordinary trait effect".into(),
            );
        }
        if kind.is_some() && !arguments.is_empty() {
            return Err("source sizing application has unexpected arguments".into());
        }
        Ok(kind)
    }
    pub(super) fn source_sizing_result(
        &mut self,
        facts: &mut Declarations,
        frame: &mut Frame,
    ) -> Result<Option<Action>, String> {
        let Some(kind) = Self::source_sizing_kind(facts, &frame.goal)? else {
            return Ok(None);
        };
        // A pending or selected source candidate remains authoritative here.
        if frame.deferred || frame.partial.is_some() || frame.selected.is_some() {
            return Ok(None);
        }
        let goal = match frame
            .inputs
            .context
            .resolved_predicate(&mut facts.types, &frame.goal)
        {
            Ok(goal) => goal,
            Err(crate::types::infer::Error::Unresolved) => {
                return Ok(Some(Action::Complete(Decision::Deferred)));
            }
            Err(error) => return Err(inference(error)),
        };
        frame.require_nominal_formed(facts, goal.ty)?;
        if let Some(holds) = self.negative_parameter_sizing(facts, kind, goal.ty)? {
            if !holds {
                return Ok(Some(Action::Complete(Decision::Absent)));
            }
            let debt = (kind == LangItem::Sized, kind == LangItem::MetaSized);
            return self.complete_source_sizing(facts, frame, goal, debt);
        }
        let shape = self.builtin_shape(facts, frame, goal.ty)?;
        if !shape.well_formed {
            return Ok(Some(Action::Complete(Decision::Absent)));
        }
        let valid = if kind == LangItem::Sized || shape.sized {
            shape.sized
        } else {
            match facts
                .types
                .kind(goal.ty)
                .ok_or("missing source sizing subject")?
            {
                Kind::Never
                | Kind::Primitive(_)
                | Kind::Reference { .. }
                | Kind::Pattern { .. }
                | Kind::Pointer { .. }
                | Kind::Function { .. }
                | Kind::FunctionItem { .. }
                | Kind::Closure { .. }
                | Kind::Future { .. }
                | Kind::Slice(_)
                | Kind::Dynamic { .. } => true,
                // `PointeeSized` holds for every type, as in rustc 1.98.
                Kind::Foreign(_)
                | Kind::Array { .. }
                | Kind::ArrayTerm { .. }
                | Kind::Parameter(_)
                | Kind::Projection(_)
                | Kind::Opaque(..)
                    if kind == LangItem::PointeeSized =>
                {
                    true
                }
                // A rigid parameter in a negative coherence query with
                // premises was decided above; any other is not `MetaSized`.
                Kind::Foreign(_)
                | Kind::Array { .. }
                | Kind::ArrayTerm { .. }
                | Kind::Parameter(_) => false,
                // A normalized projection is rigid. Its own item bounds were
                // consulted before this fallback and did not prove the goal.
                Kind::Projection(_) if Self::normalized_inputs(frame) => false,
                Kind::Projection(_) | Kind::Opaque(..) => {
                    return Err(
                        "source sizing requires a proved normalized or declared subject".into(),
                    );
                }
                Kind::Adt(..) | Kind::Tuple(_) => {
                    match facts
                        .aggregates
                        .prepare_physical_metadata(&mut facts.types, goal.ty)?
                    {
                        MetadataPreparation::Fields(item) => {
                            return Ok(Some(Action::Wait(super::Dependency::Fields(item))));
                        }
                        MetadataPreparation::Unresolved(_) | MetadataPreparation::Ready => {}
                    }
                    match self.source_tail_sizing(facts, frame, goal, kind)? {
                        ControlFlow::Break(action) => return Ok(Some(action)),
                        ControlFlow::Continue(valid) => valid,
                    }
                }
            }
        };
        if !valid {
            return Ok(Some(Action::Complete(Decision::Absent)));
        }
        match kind {
            LangItem::Sized | LangItem::MetaSized | LangItem::PointeeSized => {
                self.complete_source_sizing(facts, frame, goal, (shape.sized, false))
            }
            _ => Err("invalid source sizing fallback kind".into()),
        }
    }
    /// A `MetaSized` premise is retained like a Sized debt, so its node is
    /// never debt-free and never pays an equality-only negative.
    fn complete_source_sizing(
        &mut self,
        facts: &Declarations,
        frame: &mut Frame,
        goal: Predicate,
        (sized, meta_sized): (bool, bool),
    ) -> Result<Option<Action>, String> {
        let node = self.source_builtin_node(frame, goal, sized || meta_sized);
        let selected =
            self.query
                .push_source_sizing(facts, node, frame.scope, frame.first..frame.first)?;
        if meta_sized {
            self.query.steps = self.query.steps.saturating_add(1);
            self.query
                .negative_sizing
                .as_mut()
                .ok_or("negative coherence MetaSized proof lost its premises")?
                .record(selected)?;
        }
        Ok(Some(Action::Complete(Decision::Proven(selected))))
    }
    /// A rigid parameter's sizing `kind` in a negative coherence query with
    /// premises, which only the other header's premises decide; `None` for
    /// every other subject and query.
    fn negative_parameter_sizing(
        &mut self,
        facts: &Declarations,
        kind: LangItem,
        ty: crate::types::TypeId,
    ) -> Result<Option<bool>, &'static str> {
        if !self.caller.retain_sized || !matches!(facts.types.kind(ty), Some(Kind::Parameter(_))) {
            return Ok(None);
        }
        let Some(sizing) = self.query.negative_sizing.as_deref() else {
            return Ok(None);
        };
        let (holds, work) = sizing.parameter(kind, ty)?;
        self.query.steps = self.query.steps.saturating_add(work);
        Ok(Some(holds))
    }
    /// A builtin requirement of a selected impl. A rigid parameter's `Sized`
    /// in a negative coherence query is decided by the other header's premises.
    pub(super) fn builtin_requirement(
        &mut self,
        facts: &mut Declarations,
        requirement: Predicate,
    ) -> Result<bool, &'static str> {
        if requirement.bound == crate::types::parameters::predicates::Bound::Sized
            && let Some(sized) =
                self.negative_parameter_sizing(facts, LangItem::Sized, requirement.ty)?
        {
            return Ok(sized);
        }
        self.query.builtins.prove(facts, requirement)
    }
    /// A `Bound::Sized` goal's rigid parameter in a negative coherence query.
    pub(super) fn negative_parameter_sized(
        &mut self,
        facts: &Declarations,
        goal: Predicate,
    ) -> Result<Option<bool>, &'static str> {
        if goal.bound != crate::types::parameters::predicates::Bound::Sized {
            return Ok(None);
        }
        self.negative_parameter_sizing(facts, LangItem::Sized, goal.ty)
    }
    /// After input normalization, an assumption over another projection may be
    /// incomparable with this one. A sizing marker has no items or arguments, so
    /// a rigid projection's own item bounds prove it as well: its written
    /// bounds, the implicit `MetaSized` of a `?Sized` result or a declared
    /// `Sized`. `PointeeSized` holds for every type. Anything else stays deferred.
    pub(super) fn rigid_sizing(
        &mut self,
        facts: &mut Declarations,
        frame: &mut Frame,
    ) -> Result<Option<Action>, String> {
        if Self::source_sizing_kind(facts, &frame.goal)?.is_none() {
            return Ok(None);
        }
        let rigid = match frame
            .inputs
            .context
            .resolved(&mut facts.types, frame.goal.subject())
        {
            Ok(ty) => matches!(facts.types.kind(ty), Some(Kind::Projection(_))),
            Err(crate::types::infer::Error::Unresolved) => false,
            Err(error) => return Err(inference(error)),
        };
        if !rigid {
            return Ok(None);
        }
        if let Some(action) = self.derive_result(facts, frame)? {
            return Ok(Some(action));
        }
        Ok(self
            .source_sizing_result(facts, frame)?
            .filter(|action| !matches!(action, Action::Complete(Decision::Absent))))
    }
    /// The physical shape of a source builtin goal's subject, whose nominal
    /// formation the frame already retains as child goals. Normalized inputs
    /// keep their rigid projections and the caller's lifetimes, so the builtin
    /// proof must not normalize them again.
    pub(super) fn builtin_shape(
        &mut self,
        facts: &mut Declarations,
        frame: &Frame,
        ty: crate::types::TypeId,
    ) -> Result<crate::types::parameters::shape::Properties, &'static str> {
        if Self::normalized_inputs(frame) {
            self.query.builtins.normalized_physical_shape(facts, ty)
        } else {
            self.query.builtins.physical_shape(facts, ty)
        }
    }
    /// The frame's inputs are normalized under this search's environment: it
    /// normalized them itself ([`Self::own_inputs_normalized`]), or they arrived
    /// normalized, from a parent normalization (`Frame::child`) or from a
    /// borrowed query in normalized or projection mode. A shape walk needs only
    /// that their remaining projections are rigid, which both guarantee.
    fn normalized_inputs(frame: &Frame) -> bool {
        frame.normalized_inputs.is_some() || Self::own_inputs_normalized(frame)
    }
    /// `advance_inputs` replaced every projected input root of this frame with
    /// its normalized value. Unlike [`Self::normalized_inputs`] this excludes
    /// inputs that arrived normalized: the builtin proof skips its own
    /// normalization only on the record of the normalization this frame ran.
    pub(super) fn own_inputs_normalized(frame: &Frame) -> bool {
        frame
            .normalization
            .as_ref()
            .is_some_and(|normalization| normalization.original_inputs().is_some())
    }
    /// Reuse the same actual terminal as metadata normalization. In particular,
    /// an opaque external terminal has unit metadata without becoming `MetaSized`.
    fn source_tail_sizing(
        &mut self,
        facts: &Declarations,
        frame: &mut Frame,
        goal: Predicate,
        requested: LangItem,
    ) -> Result<ControlFlow<Action, bool>, String> {
        let mut tail = goal.ty;
        loop {
            frame.inputs.context.account_work(1).map_err(inference)?;
            self.charge(&mut frame.inputs.context)?;
            match crate::abi::metadata::declared_step(&facts.types, &facts.aggregates, tail)? {
                crate::abi::metadata::Step::Tail(child) => tail = child,
                crate::abi::metadata::Step::Complete(Metadata::Thin) => {
                    return Ok(ControlFlow::Continue(
                        !matches!(facts.types.kind(tail), Some(Kind::Foreign(_)))
                            || requested == LangItem::PointeeSized,
                    ));
                }
                crate::abi::metadata::Step::Complete(Metadata::Length | Metadata::VTable(_)) => {
                    return Ok(ControlFlow::Continue(true));
                }
                crate::abi::metadata::Step::Complete(Metadata::Unknown(_)) => {
                    return Self::source_tail_condition(frame, tail, goal.bound);
                }
            }
        }
    }
    pub(super) fn complete_source_builtin(
        &mut self,
        frame: &mut Frame,
        goal: Predicate,
        is_sized: bool,
    ) -> Action {
        let node = self.source_builtin_node(frame, goal, is_sized);
        let selected = self.query.push(node, frame.scope, frame.first..frame.first);
        Action::Complete(Decision::Proven(selected))
    }
    fn source_builtin_node(&self, frame: &mut Frame, goal: Predicate, is_sized: bool) -> Node {
        let mut sized = Vec::new();
        if is_sized && self.caller.retain_sized {
            sized.push(goal.ty);
        }
        let end = frame.available_region();
        Node {
            debt_free: false,
            cycle_dependent: false,
            provisional_size: false,
            goal,
            source: Some(Source::Builtin),
            children: frame
                .storage
                .as_mut()
                .map_or_else(Vec::new, |storage| std::mem::take(&mut storage.children)),
            first: frame.first,
            end,
            equalities: std::mem::take(&mut frame.inputs.context.region_equalities),
            outlives: Vec::new(),
            type_outlives: Vec::new(),
            sized,
        }
    }
}
