//! Projected builtin premises use the same normalization stack and lifetime evidence.
use super::{Action, Decision, Declarations, Frame, Node, Search, inference};
use crate::types::{RegionId, TypeId, parameters::predicates::Bound};

#[derive(Default)]
pub(super) struct Requirements {
    pub cursor: usize,
    /// Monotone numeric-domain certificates, not canonical proof nodes. A
    /// compatible inference equation cannot turn a numeric root into an unsized
    /// type. Stop at the first debt that has no such current certificate.
    pub numeric_cursor: usize,
    pub type_outlives: Vec<(TypeId, RegionId)>,
    pub sized: Vec<TypeId>,
}

impl super::Trial {
    pub(super) fn classify_materialization(
        &mut self,
        facts: &Declarations,
        requirements: &mut Requirements,
    ) -> Result<(), String> {
        self.inputs.context.account_work(1).map_err(inference)?;
        if self.instance.quantified.is_some()
            || !self.instance.regions.is_empty()
            || !self.instance.outlives.is_empty()
            || !self.pending.is_empty()
        {
            return Ok(());
        }
        while let Some(goal) = self.instance.goals.get(requirements.numeric_cursor) {
            self.inputs.context.account_work(1).map_err(inference)?;
            let sizing = if let Some(bound) = goal.builtin_bound() {
                bound == Bound::Sized
            } else if self.wakeups.contains(requirements.numeric_cursor) {
                Search::source_sizing_kind(facts, goal)?.is_some()
            } else {
                requirements.numeric_cursor += 1;
                continue;
            };
            if sizing {
                let context = &mut self.inputs.context;
                let numeric = matches!(
                    context
                        .unresolved_domain(goal.subject())
                        .map_err(inference)?,
                    Some(crate::types::infer::Domain::Integer | crate::types::infer::Domain::Float)
                );
                let primitive = matches!(
                    context.head(&facts.types, goal.subject()).map_err(inference)?,
                    Some(crate::types::infer::Constructor::Primitive(value))
                        if value != crate::types::Primitive::Str
                );
                if !numeric && !primitive {
                    return Ok(());
                }
            } else {
                return Ok(());
            }
            // Only actual numeric sizing watches may remain. Every other
            // child has already completed its own canonical proof frontier.
            requirements.numeric_cursor += 1;
        }
        Ok(())
    }

    pub(super) fn classify_waiting_materialization(
        &mut self,
        facts: &mut Declarations,
    ) -> Result<(), String> {
        use crate::{resolve::body_origin::DeclarationOrigin, support::bindings::LangItem};
        let role = facts
            .source_bindings
            .as_deref()
            .map(|catalog| catalog.role(LangItem::From))
            .transpose()?
            .flatten();
        let Some(role) = role else { return Ok(()) };
        let mut retained = self.requirements.take();
        let mut scratch = Requirements::default();
        let requirements = retained.as_deref_mut().unwrap_or(&mut scratch);
        let result = self.classify_materialization(facts, requirements);
        if result.is_ok() && retained.is_none() && scratch.numeric_cursor != 0 {
            // This optional cold payload belongs to a source-role query. Reuse
            // its actual producer's sticky ledger and conservatively retain the
            // scratch charge; this supplies no header or selection permission.
            let paid = match role.origin {
                DeclarationOrigin::Source(_) => crate::sema::header_attributes::source_payments(
                    facts.must_use_contracts.as_deref(),
                )
                .and_then(|payments| {
                    payments
                        .charge(1, size_of::<Requirements>())
                        .map_err(crate::resolve::package::Error::unlocated)
                }),
                DeclarationOrigin::Imported(_) => {
                    crate::sema::crates::charge_source_headers(facts, 1, size_of::<Requirements>())
                }
            };
            paid.map_err(|error| error.to_string())?;
            retained = Some(Box::new(scratch));
        }
        self.requirements = retained;
        result
    }
}

impl Search<'_> {
    pub(super) fn complete_builtin(
        &mut self,
        facts: &mut Declarations,
        frame: &mut Frame,
    ) -> Result<Action, String> {
        let goal = match frame
            .inputs
            .context
            .resolved_predicate(&mut facts.types, &frame.goal)
        {
            Ok(goal) => goal,
            Err(crate::types::infer::Error::Unresolved) => {
                return Ok(Action::Complete(Decision::Deferred));
            }
            Err(error) => return Err(inference(error)),
        };
        if let Bound::Auto(
            marker @ (crate::types::auto_traits::AutoTrait::Freeze
            | crate::types::auto_traits::AutoTrait::UnsafeUnpin),
        ) = goal.bound
        {
            return Err(format!(
                "internal {} rule requires an actual source trait application",
                marker.name()
            ));
        }
        let mut type_outlives = Vec::new();
        let mut sized = Vec::new();
        let mut provisional_size = false;
        if let Bound::Outlives(region) = goal.bound {
            type_outlives.push((goal.ty, region));
        } else {
            let valid = if let Bound::Auto(marker) = goal.bound {
                match self.auto_condition(
                    facts,
                    frame,
                    crate::types::auto_traits::Obligation {
                        ty: goal.ty,
                        marker,
                    },
                )? {
                    std::ops::ControlFlow::Break(action) => return Ok(action),
                    std::ops::ControlFlow::Continue(value) => value,
                }
            } else if goal.bound == Bound::Sized
                && (facts.types.contains_projected_sizes(goal.ty) == Some(true)
                    || !facts.types.nominal_sizes_sealed()
                        && facts.types.contains_nominals(goal.ty) == Some(true))
            {
                match self.storage_condition(facts, frame, goal.ty)? {
                    std::ops::ControlFlow::Break(action) => return Ok(action),
                    std::ops::ControlFlow::Continue((sized, provisional)) => {
                        provisional_size = provisional;
                        sized
                    }
                }
            } else {
                self.query.builtins.prove(facts, goal)?
            };
            if !valid {
                return Ok(Action::Complete(Decision::Absent));
            }
            if goal.bound == Bound::Sized && self.caller.retain_sized {
                sized.push(goal.ty);
            }
        }
        let mut children = frame
            .storage
            .as_mut()
            .map_or_else(Vec::new, |storage| std::mem::take(&mut storage.children));
        if let Some(autos) = &mut frame.autos {
            children.append(&mut autos.children);
        }
        let node = self.query.push(
            Node {
                debt_free: false,
                cycle_dependent: false,
                provisional_size,
                goal,
                source: Some(super::Source::Builtin),
                children,
                first: frame.first,
                end: frame.available_region(),
                equalities: std::mem::take(&mut frame.inputs.context.region_equalities),
                outlives: Vec::new(),
                type_outlives,
                sized,
            },
            frame.scope,
            frame.first..frame.first,
        );
        Ok(Action::Complete(Decision::Proven(node)))
    }
}
