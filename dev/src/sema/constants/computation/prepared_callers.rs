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
pub(crate) struct BodyCaller {
    key: catalog::CallerKey,
    publication: catalog::Publication,
    aliases: Bound,
    reservation: Reservation,
}
impl BodyCaller {
    pub(crate) fn needed(
        facts: &Declarations,
        instance: Instance,
        result: TypeId,
    ) -> Result<bool, &'static str> {
        if !super::source_result(&facts.types, result) {
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
                        || facts.types.needs_region_context(ty) != Some(false)
                    {
                        return Ok(false);
                    }
                }
                Argument::Const(id) => {
                    if !matches!(facts.types.const_kind(id), Some(ConstKind::Value(_))) {
                        return Ok(false);
                    }
                }
            }
        }
        Ok(open
            || facts
                .definitions
                .get(instance.definition.0 as usize)
                .copied()
                .flatten()
                .is_some_and(|ty| facts.types.contains_projections(ty) == Some(true)))
    }
    pub(crate) fn new(
        facts: &mut Declarations,
        source: NodeKey,
        enclosing: CallerInstance,
        universals: u32,
        equalities: impl IntoIterator<Item = Pair>,
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
        let mut equations = RegionEquations::default();
        let work = equations.extend(equalities)?;
        let mut aliases = Bound::close_projection_regions(1, 0, std::collections::HashMap::new())?;
        aliases.exchange_projection_equations(&mut equations)?;
        facts.body_work.canonical_steps += work;
        Ok(Self {
            key,
            publication,
            aliases,
            reservation: Reservation::default(),
        })
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
        let selected = dispatcher.constant(facts, input)?;
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
        let original = facts
            .initializer_index(selected.definition)
            .and_then(|n| facts.initializers.get(n))
            .ok_or("source operand has no original initializer")?
            .semantic_source();
        let seal = Seal::prepare(facts, dispatcher, selected, result)?;
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
