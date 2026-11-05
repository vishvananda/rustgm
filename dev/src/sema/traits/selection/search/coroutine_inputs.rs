//! Genuine Future evidence inherits the actual defining application's input proof.
use super::{Action, Decision, Declarations, Frame, Node, Search, Source, inference};
use crate::{
    resolve::model::DefId,
    sema::coroutine_methods,
    support::bindings::LangItem,
    types::{GenericArgs, Kind, infer},
};

impl Search<'_> {
    pub(super) fn source_future_goal(facts: &Declarations, frame: &Frame) -> Result<bool, String> {
        let Some((item, _)) = frame.goal.trait_arguments() else {
            return Ok(false);
        };
        Ok(coroutine_methods::future_role(facts)? == Some(DefId(item.0)))
    }

    pub(super) fn source_future_result(
        &mut self,
        facts: &mut Declarations,
        frame: &mut Frame,
    ) -> Result<Option<Action>, String> {
        if !Self::source_future_goal(facts, frame)? {
            return Ok(None);
        }
        let goal = match frame
            .inputs
            .context
            .resolved_predicate(&mut facts.types, &frame.goal)
        {
            Ok(goal) => goal,
            Err(infer::Error::Unresolved | infer::Error::Normalization) => {
                return Ok(Some(Action::Complete(Decision::Deferred)));
            }
            Err(error) => return Err(inference(error)),
        };
        if !matches!(facts.types.kind(goal.ty), Some(Kind::Future { .. })) {
            return Ok(None);
        }
        facts.checked_future_definition(goal.ty)?;
        let (application, effect) = goal
            .bound
            .trait_parts()
            .ok_or("future evidence requires its actual trait application")?;
        let application = facts
            .types
            .trait_application(application)
            .ok_or("future evidence lost its actual trait application")?;
        if application.arguments != GenericArgs::EMPTY
            || effect != crate::support::constness::Constness::Ordinary
        {
            return Ok(None);
        }
        let catalog = facts
            .source_bindings
            .as_deref()
            .ok_or("future evidence lost its actual source catalog")?;
        catalog
            .validated_language(LangItem::Future)?
            .ok_or("future evidence requires its completed actual trait")?;
        frame.require_nominal_formed(facts, goal.ty)?;
        if frame.selected.is_some() {
            return Ok(None);
        }
        if frame.deferred || frame.partial.is_some() {
            return Ok(Some(Action::Complete(Decision::Deferred)));
        }
        let end = frame.available_region();
        let node = self.query.push(
            Node {
                debt_free: false,
                cycle_dependent: false,
                provisional_size: false,
                goal,
                source: Some(Source::Future { future: goal.ty }),
                children: Vec::new(),
                first: frame.first,
                end,
                equalities: std::mem::take(&mut frame.inputs.context.region_equalities),
                outlives: Vec::new(),
                type_outlives: Vec::new(),
                sized: Vec::new(),
            },
            frame.scope,
            end..end,
        );
        Ok(Some(Action::Complete(Decision::Proven(node))))
    }
}

impl Search<'_> {
    pub(super) fn begin_future_formation(
        &mut self,
        facts: &mut Declarations,
        frame: &mut Frame,
        future: crate::types::TypeId,
        depth: u16,
        select: bool,
        cached: Option<usize>,
    ) -> Result<bool, String> {
        use super::member_conditions::{Entry, Formation, Key, Purpose};
        facts.checked_future_definition(future)?;
        if depth != 0 {
            return Err("future defining inputs under nested callable binders require scoped formation transport".into());
        }
        let Some(Kind::Future { arguments, .. }) = facts.types.kind(future) else {
            return Err("future formation requires its actual full application".into());
        };
        let count = facts
            .types
            .arguments(arguments)
            .ok_or("future formation lost its complete arguments")?
            .len();
        facts.charge_caller_inputs(
            count.saturating_add(1),
            size_of::<Formation>()
                .saturating_add(count.saturating_mul(size_of::<(crate::types::TypeId, u16)>())),
        )?;
        let mut nominals = Vec::with_capacity(if cached.is_none() { count } else { 0 });
        let requirements = if cached.is_none() {
            let requirements = coroutine_methods::inputs(facts, future)?;
            for argument in facts
                .types
                .arguments(arguments)
                .ok_or("future formation lost its complete arguments")?
            {
                if let crate::types::arguments::Argument::Type(ty) = argument {
                    nominals.push((ty, depth));
                }
            }
            self.query
                .families
                .insert(Key::Nominal(future, depth, frame.scope), Entry::Active);
            requirements
        } else {
            crate::sema::projections::families::Requirements {
                clauses: Vec::new(),
                regions: Vec::new(),
                steps: 0,
            }
        };
        frame
            .inputs
            .context
            .account_work(requirements.steps.saturating_add(count))
            .map_err(inference)?;
        frame.formation = Some(Box::new(Formation {
            purpose: if select {
                Purpose::Select(future)
            } else {
                Purpose::Nominal(future, depth)
            },
            requirements,
            nominals,
            parent: cached.map(super::frames::Chosen::Positive),
            cursor: 0,
            children: Vec::new(),
            universes: crate::support::hash::HashMap::default(),
            stopped: None,
            registered: cached.is_none(),
            finished: cached.is_some() && select,
        }));
        Ok(cached.is_none())
    }
}
