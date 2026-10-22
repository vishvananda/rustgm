//! Real body callers retain their source publication and append debts before proof.
use super::{
    CallerInstance, Declarations, Instance, NodeKey, SourceComputation, TypeId, catalog,
    seals::Seal,
};
use crate::{
    sema::{dispatch::Dispatcher, traits::obligations::Reservation},
    types::{
        arguments::{Argument, ConstKind},
        outlives::Pair,
        substitute::{Bound, RegionEquations},
    },
};
pub(crate) enum PreparedConstant {
    Selected(Instance),
    Source(SourceComputation),
}
pub(crate) struct BodyCaller<'a> {
    inputs: Option<crate::sema::callee_premises::Inputs<'a>>,
    key: catalog::CallerKey,
    publication: catalog::Publication,
    aliases: Bound,
    reservation: Reservation,
}
impl<'a> BodyCaller<'a> {
    pub(crate) fn needed(
        facts: &Declarations,
        instance: Instance,
        result: TypeId,
    ) -> Result<bool, &'static str> {
        let canonical_result = super::source_result(&facts.types, result);
        if !canonical_result && !super::source_result_candidate(&facts.types, result) {
            return Ok(false);
        }
        let mut open = false;
        for argument in facts
            .types
            .arguments(instance.arguments)
            .ok_or("missing associated operand arguments")?
        {
            match argument {
                Argument::Lifetime(region) => open |= region != crate::types::RegionId::STATIC,
                Argument::Type(ty) => {
                    if facts.types.contains_parameters(ty) != Some(false)
                        || facts.types.contains_projections(ty) != Some(false)
                    {
                        return Ok(false);
                    }
                    open |= facts.types.needs_region_context(ty) == Some(true);
                }
                Argument::Const(id) => {
                    if !matches!(facts.types.const_kind(id), Some(ConstKind::Value(_))) {
                        return Ok(false);
                    }
                }
            }
        }
        if !open && !canonical_result {
            return Ok(false);
        }
        Ok(open
            || facts
                .definitions
                .get(instance.definition.0 as usize)
                .copied()
                .flatten()
                .is_some_and(|ty| facts.types.contains_projections(ty) == Some(true)))
    }
    #[cfg(test)]
    pub(crate) fn new(
        facts: &mut Declarations,
        source: NodeKey,
        enclosing: CallerInstance,
        universals: u32,
        equalities: impl IntoIterator<Item = Pair>,
    ) -> Result<Self, &'static str> {
        Self::new_using(facts, source, enclosing, universals, equalities, None)
    }
    pub(crate) fn with_inputs(
        facts: &mut Declarations,
        source: NodeKey,
        enclosing: CallerInstance,
        universals: u32,
        equalities: impl IntoIterator<Item = Pair>,
        inputs: crate::sema::callee_premises::Inputs<'a>,
    ) -> Result<Self, &'static str> {
        if inputs.source != source || inputs.universals != universals {
            return Err("source caller inputs belong to another body or binder");
        }
        Self::new_using(
            facts,
            source,
            enclosing,
            universals,
            equalities,
            Some(inputs),
        )
    }
    fn new_using(
        facts: &mut Declarations,
        source: NodeKey,
        enclosing: CallerInstance,
        universals: u32,
        equalities: impl IntoIterator<Item = Pair>,
        inputs: Option<crate::sema::callee_premises::Inputs<'a>>,
    ) -> Result<Self, &'static str> {
        let catalog = &facts
            .source_computations
            .as_deref()
            .ok_or("body source caller was not published")?
            .0;
        let (mut key, publication) = catalog.original(source)?;
        publication.validate(facts, source)?;
        key.enclosing = enclosing;
        if !matches!(enclosing, CallerInstance::Original)
            && !super::namespaces::contains(facts, key)
        {
            return Err("specialized source caller requires its owned namespace transport");
        }
        if matches!(enclosing, CallerInstance::Original) && key.universals != universals {
            return Err("source caller body binder changed from its completed publication");
        }
        key.universals = universals;
        let mut work = 0;
        let equations = if let Some(inputs) = inputs {
            inputs.prepare(facts)?;
            inputs.aliases(facts, equalities, &mut work)
        } else {
            let mut equations = RegionEquations::default();
            equations
                .extend_counted(equalities, &mut work)
                .map(|()| equations)
        };
        facts.body_work.canonical_steps += work;
        let mut equations = equations?;
        let mut aliases =
            Bound::close_projection_regions(1, 0, crate::support::hash::HashMap::default())?;
        aliases.exchange_projection_equations(&mut equations)?;
        Ok(Self {
            inputs,
            key,
            publication,
            aliases,
            reservation: Reservation::default(),
        })
    }
    fn original_input(
        &mut self,
        facts: &mut Declarations,
        dispatcher: &mut Dispatcher,
        input: Instance,
    ) -> Result<Instance, String> {
        dispatcher.retain_source_aliases(facts, input, &mut self.aliases)?;
        let before = self.aliases.steps;
        let result = (|| {
            let available =
                SourceComputation::with_argument_regions(facts, input, |_, regions| {
                    for &region in regions {
                        if !self.aliases.projection_input_available(region)? {
                            return Ok::<_, &'static str>(false);
                        }
                    }
                    Ok(true)
                })?;
            // A declaration-inferred input can have an exact ordinary AST value
            // without belonging to this body's source namespace. Select it first;
            // absence of anchors must never manufacture source authority.
            if available {
                input.substitute(&mut facts.types, &mut self.aliases)
            } else {
                Ok(input)
            }
        })();
        facts.body_work.canonical_steps += self.aliases.steps - before;
        result.map_err(String::from)
    }
    pub(crate) fn prepare(
        &mut self,
        facts: &mut Declarations,
        dispatcher: &mut Dispatcher,
        input: Instance,
        actual_result: TypeId,
    ) -> Result<PreparedConstant, String> {
        // Retain the current selection's actual proof before choosing an
        // execution route. A prior target lookup alone is not caller evidence.
        // Input roots belong to this original Contract. Close those exact
        // identities before selection opens the receiving dispatch universals.
        let open_types = facts
            .types
            .arguments(input.arguments)
            .ok_or("missing source arguments")?
            .any(|argument| {
                matches!(argument, Argument::Type(ty)
                if facts.types.needs_region_context(ty) == Some(true))
            });
        let canonical_input = if open_types {
            self.original_input(facts, dispatcher, input)?
        } else {
            input
        };
        let selected = dispatcher.constant(facts, canonical_input)?;
        if !Self::needed(facts, selected, actual_result)?
            || crate::sema::values::Reader::new(facts)
                .associated(selected)
                .is_some()
        {
            return Ok(PreparedConstant::Selected(selected));
        }
        // Newly inferred selector aliases already owned by this dispatcher may
        // be retained; original equation mappings must never be overwritten.
        let before = self.aliases.steps;
        dispatcher.retain_source_aliases(facts, selected, &mut self.aliases)?;
        let canonical = selected.substitute(&mut facts.types, &mut self.aliases)?;
        let result = self.aliases.apply(&mut facts.types, actual_result)?;
        facts.body_work.canonical_steps += self.aliases.steps - before;
        let selected = dispatcher.source_instance(facts, canonical)?;
        let result = dispatcher.source_result(facts, result)?;
        // Real source equalities can close every argument (notably an explicit
        // static use). That canonical Instance keeps the ordinary concrete gate.
        if !Self::needed(facts, selected, result)? {
            return Ok(PreparedConstant::Selected(selected));
        }
        let (selected, paid) = if let Some(inputs) = self.inputs {
            super::paid_inputs::select(
                facts,
                dispatcher,
                self.key,
                self.publication,
                inputs,
                selected,
            )?
        } else {
            (selected, None)
        };
        let original = facts
            .initializer_index(selected.definition)
            .and_then(|n| facts.initializers.get(n))
            .ok_or("source operand has no original initializer")?
            .semantic_source();
        let seal = Seal::prepare(facts, dispatcher, selected, result, None)?.with_paid(paid)?;
        let requested = seal.result();
        dispatcher.reserve_pattern(facts, self.key.source, &mut self.reservation)?;
        let catalog = &mut facts
            .source_computations
            .as_deref_mut()
            .ok_or("missing body source caller catalog")?
            .0;
        let caller = catalog.caller(self.key, self.publication)?;
        let witness = SourceComputation(catalog.retain_use(catalog::Use {
            caller,
            input,
            selected,
            original,
            actual_result,
            requested,
            seal: Some(seal),
        })?);
        facts.constant_work.source_context_bytes = catalog.retained_bytes();
        witness.validate(facts, requested)?;
        Ok(PreparedConstant::Source(witness))
    }
}
