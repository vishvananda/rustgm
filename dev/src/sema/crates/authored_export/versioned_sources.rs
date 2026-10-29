//! The existing declaration census closes real feature-attribute transport coverage.
use super::{Declarations, DefId, Report, Stage};
use crate::{
    resolve::{model::DefKind, unit::NodeKey},
    support::{hash::HashSet, retained},
    types::parameters::clauses::Clause,
};
pub(super) struct Coverage {
    pending: Option<HashSet<NodeKey>>,
}
impl Coverage {
    pub(super) fn new(
        report: &Report,
        stage: &mut Stage,
        enabled: bool,
    ) -> Result<Self, &'static str> {
        if !enabled {
            return Ok(Self { pending: None });
        }
        let producer = report
            .lint_producer()
            .ok_or("feature transport lost its actual attribute producer")?;
        let keys = producer.feature_bound_keys();
        let count = keys
            .size_hint()
            .1
            .ok_or("feature transport lacks its actual inventory bound")?;
        stage.charge(count.saturating_add(1))?;
        stage.reserve(retained::new_map::<NodeKey, ()>(count))?;
        Ok(Self {
            pending: Some(keys.collect()),
        })
    }
    pub(super) fn observe(
        &mut self,
        report: &Report,
        facts: &Declarations,
        owner: DefId,
        stage: &mut Stage,
    ) -> Result<(), &'static str> {
        let Some(pending) = &mut self.pending else {
            return Ok(());
        };
        let definition = &report.resolution.definitions[owner.0 as usize];
        if !matches!(
            definition.kind,
            DefKind::Trait | DefKind::Function | DefKind::SelfType
        ) {
            return Ok(());
        }
        if definition.kind == DefKind::SelfType
            && facts
                .associated
                .owner(owner)
                .and_then(|id| facts.associated.get(id))
                .is_some_and(|owner| {
                    matches!(owner.kind, crate::sema::associated::OwnerKind::Trait { .. })
                })
        {
            return Ok(());
        }
        let Some(key) = definition.node else {
            return Ok(());
        };
        let present = pending.contains(&key);
        stage.charge(
            facts
                .generics
                .declared_predicates(owner)
                .len()
                .saturating_mul(3)
                .saturating_add(1),
        )?;
        let has_clauses = facts
            .generics
            .declared_predicates(owner)
            .iter()
            .any(|predicate| matches!(predicate.value, Clause::UnstableFeature(_)));
        if !present {
            return if has_clauses {
                Err("source feature requirement lost its actual attribute owner")
            } else {
                Ok(())
            };
        }
        let actual = report
            .lint_producer()
            .ok_or("feature transport lost its source producer")?
            .declaration_annotation(&report.package, key)
            .map_err(|error| error.message)?
            .filter(|annotation| annotation.bound.is_some())
            .ok_or("feature transport lost its authenticated annotation")?;
        if !facts
            .generics
            .declared_predicates(owner)
            .iter()
            .filter(|predicate| matches!(predicate.value, Clause::UnstableFeature(_)))
            .map(|predicate| {
                if let Clause::UnstableFeature(id) = predicate.value {
                    facts.types.feature_name(id)
                } else {
                    None
                }
            })
            .eq(actual.bounds.iter().map(|name| Some(name.as_ref())))
        {
            return Err("source feature requirements differ from their current typed producer");
        }
        pending.remove(&key);
        Ok(())
    }
    pub(super) fn finish(self) -> Result<(), &'static str> {
        if self.pending.is_some_and(|pending| !pending.is_empty()) {
            return Err("feature transport omitted an actual trait, function or impl annotation");
        }
        Ok(())
    }
}

impl Stage {
    pub(super) fn install_private_symbols(
        &mut self,
        facts: &mut Declarations,
        identity: &crate::support::crates::CrateIdentity,
    ) -> Result<(), &'static str> {
        let Some(methods) = self.inputs.selected_methods.as_deref() else {
            return Ok(());
        };
        let count = methods.len();
        self.reserve(count.saturating_mul(size_of::<DefId>()))?;
        let definitions = self
            .inputs
            .selected_methods
            .as_deref()
            .ok_or("source method index disappeared")?
            .keys()
            .copied()
            .collect::<Vec<_>>();
        for definition in definitions {
            self.charge(3)?;
            let declaration = self.declaration(definition)?;
            let body = self
                .functions
                .bodies
                .get(&definition)
                .copied()
                .ok_or("private method lost its real reserved original")?;
            let growth = 4 * (identity.name().len() + identity.disambiguator().len() + 64);
            self.charge(growth)?;
            self.reserve(growth.saturating_add(retained::new_map::<DefId, String>(1)))?;
            let symbol = identity.private_original_symbol(body, declaration);
            let linkage = facts.crates.get_or_insert_default();
            if linkage
                .symbols
                .get(&definition)
                .is_some_and(|old| *old != symbol)
            {
                return Err("private original changed its canonical source symbol");
            }
            linkage.symbols.insert(definition, symbol);
        }
        Ok(())
    }
}
