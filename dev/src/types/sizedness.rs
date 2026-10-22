//! Nominal Sized conditions are separate from declaration identity and value layout.
#[cfg(test)]
#[path = "../../tests/invariants/late_nominal_sizes.rs"]
mod late_tests;
mod schemas;
use super::{GenericArgs, ItemId, Kind, Property, Store, TypeId};
use std::collections::HashMap;

/// A declaration's condition, interpreted only after proving its formation bounds.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rule {
    Sized,
    Unsized,
    /// The complete declaration argument slot, including preceding lifetimes/consts.
    Argument(u32),
    /// An owner-relative terminal needs substitution and caller-owned normalization.
    Projection(TypeId),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Condition {
    Sized,
    Unsized,
    Type(TypeId),
}

/// Completed rules do not change when other declarations finish.
/// Missing rules are dependencies, never provisional positive Sized answers.
#[derive(Clone, Default)]
pub struct Rules {
    entries: HashMap<ItemId, schemas::Entry>,
    projections: HashMap<(ItemId, GenericArgs), TypeId>,
    closed: HashMap<TypeId, schemas::Term>,
    pending: HashMap<ItemId, schemas::Pending>,
    pub steps: usize,
}
impl Rules {
    #[must_use]
    pub fn get(&self, item: ItemId) -> Option<Rule> {
        self.entries.get(&item).map(|entry| entry.rule)
    }
    /// Instantiate only the reduced size condition; this proves no formation bounds.
    /// # Errors
    /// Rejects incomplete declarations, invalid arguments and exhausted substitution work.
    pub fn apply(
        &mut self,
        types: &mut Store,
        item: ItemId,
        arguments: GenericArgs,
    ) -> Result<Condition, &'static str> {
        Ok(
            match self.get(item).ok_or("missing nominal sizedness rule")? {
                Rule::Sized => Condition::Sized,
                Rule::Unsized => Condition::Unsized,
                Rule::Argument(slot) => {
                    let Some(super::arguments::Argument::Type(ty)) = types
                        .arguments(arguments)
                        .and_then(|mut args| args.nth(slot as usize))
                    else {
                        return Err("sizedness argument slot is not a type");
                    };
                    Condition::Type(ty)
                }
                Rule::Projection(template) => {
                    let key = (item, arguments);
                    let ty = if let Some(&ty) = self.projections.get(&key) {
                        ty
                    } else {
                        let mut substitution =
                            super::substitute::Bound::joint(types, item, arguments)?;
                        let ty = substitution.apply(types, template)?;
                        self.charge(substitution.steps)?;
                        self.projections.insert(key, ty);
                        ty
                    };
                    Condition::Type(ty)
                }
            },
        )
    }
    fn charge(&mut self, count: usize) -> Result<(), &'static str> {
        self.steps = self
            .steps
            .checked_add(count)
            .filter(|&steps| steps <= 1_000_000)
            .ok_or("nominal sizedness work budget exceeded")?;
        Ok(())
    }
}

impl Store {
    /// Append one independent, unconditionally sized completed field record.
    /// Shared older stores retain their rules and cannot intern this new nominal.
    /// # Errors
    /// Rejects existing identities, pending/dependent rules and exhausted work
    /// before copying a shared rule inventory or publishing the new entry.
    pub(crate) fn append_sized_leaf(
        &mut self,
        definitions: &super::aggregate::Definitions,
        item: ItemId,
    ) -> Result<usize, &'static str> {
        let rules = self
            .nominal_sizes
            .as_ref()
            .ok_or("nominal rules are not sealed")?;
        if rules.get(item).is_some() || definitions.get(item).is_none() {
            return Err("late sized nominal is existing or incomplete");
        }
        if !rules.pending.is_empty() {
            return Err("sealed nominal rules contain pending declarations");
        }
        let copied = if std::sync::Arc::strong_count(rules) > 1 {
            [
                rules.entries.capacity(),
                rules.projections.capacity(),
                rules.closed.capacity(),
                rules.pending.capacity(),
            ]
            .into_iter()
            .try_fold(0usize, usize::checked_add)
            .ok_or("sizedness clone capacity overflow")?
        } else {
            0
        };
        let payload = [
            std::mem::size_of::<(ItemId, schemas::Entry)>(),
            std::mem::size_of::<((ItemId, GenericArgs), TypeId)>(),
            std::mem::size_of::<(TypeId, schemas::Term)>(),
            std::mem::size_of::<(ItemId, schemas::Pending)>(),
        ]
        .into_iter()
        .max()
        .unwrap_or(0);
        // A conservative bucket/control allowance is checked before COW allocation.
        copied
            .checked_mul(payload.saturating_add(1))
            .and_then(|n| n.checked_mul(2))
            .ok_or("sizedness clone allocation overflow")?;
        let prior = rules
            .steps
            .checked_add(copied)
            .filter(|&n| n <= 1_000_000)
            .ok_or("nominal sizedness work budget exceeded")?;
        let mut inferred = Rules::default();
        if inferred
            .prepare(self, definitions, item, |_, _, _| Ok(false))?
            .is_some()
            || inferred.entries.len() != 1
            || !inferred.pending.is_empty()
            || inferred.get(item) != Some(Rule::Sized)
        {
            return Err("late sized nominal has a dependent or conditional rule");
        }
        let total = prior
            .checked_add(inferred.steps)
            .filter(|&n| n <= 1_000_000)
            .ok_or("nominal sizedness work budget exceeded")?;
        let value = inferred
            .entries
            .remove(&item)
            .ok_or("missing inferred sized leaf")?;
        let rules = std::sync::Arc::make_mut(
            self.nominal_sizes
                .as_mut()
                .ok_or("missing sealed nominal rules")?,
        );
        rules.entries.insert(item, value);
        rules.steps = total;
        Ok(copied + inferred.steps)
    }

    pub(super) fn nominal_properties(
        &self,
        item: ItemId,
        args: GenericArgs,
    ) -> Result<super::Properties, &'static str> {
        // Identity precedes field completion. Final declaration rules
        // are sealed before consuming permanent shape proofs.
        let mut result = super::Properties::default();
        self.argument_properties(args, &mut result)?;
        result.set(Property::Nominals, true);
        if matches!(self.nominal_size_rule(item), Some(Rule::Projection(_))) {
            result.set(Property::ProjectedSizes, true);
        }
        result.set(Property::Sized, self.nominal_is_sized(item, args)?);
        Ok(result)
    }

    #[must_use]
    pub(crate) fn nominal_sizes_pending(&self) -> bool {
        self.has_nominals && self.nominal_sizes.is_none()
    }
    #[must_use]
    pub(crate) fn nominal_sizes_sealed(&self) -> bool {
        self.nominal_sizes.is_some()
    }
    pub(super) fn nominal_is_sized(
        &self,
        item: ItemId,
        args: GenericArgs,
    ) -> Result<bool, &'static str> {
        let Some(rules) = &self.nominal_sizes else {
            return Ok(true);
        };
        Ok(
            match rules.get(item).ok_or("missing nominal sizedness rule")? {
                Rule::Sized => true,
                Rule::Unsized | Rule::Projection(_) => false,
                Rule::Argument(slot) => {
                    let Some(super::arguments::Argument::Type(ty)) = self
                        .arguments(args)
                        .and_then(|mut args| args.nth(slot as usize))
                    else {
                        return Err("nominal sizedness argument is not a type");
                    };
                    self.is_sized(ty)
                        .ok_or("invalid nominal sizedness argument")?
                }
            },
        )
    }

    /// Types containing nominal identities may need field readiness during declarations.
    #[must_use]
    pub fn contains_nominals(&self, ty: TypeId) -> Option<bool> {
        self.properties
            .get(ty.0 as usize)
            .map(|properties| properties.has(Property::Nominals))
    }

    /// A storage size condition hidden behind a nominal identity needs semantic proof.
    #[must_use]
    pub fn contains_projected_sizes(&self, ty: TypeId) -> Option<bool> {
        self.properties
            .get(ty.0 as usize)
            .map(|properties| properties.has(Property::ProjectedSizes))
    }

    #[must_use]
    pub fn nominal_size_rule(&self, item: ItemId) -> Option<Rule> {
        self.nominal_sizes.as_ref()?.get(item)
    }

    /// Seal the completed declaration rules before retaining final shape proofs.
    /// Canonical structural children precede their parents, so one forward pass
    /// recomputes dependent tuple/array well-formedness and sizedness in linear work.
    /// Nominal field graphs are represented by reduced rules, not new structural edges.
    /// # Errors
    /// Rejects resealing, missing declarations and malformed canonical argument slots.
    pub fn seal_nominal_sizes(&mut self, rules: std::sync::Arc<Rules>) -> Result<(), &'static str> {
        if self.nominal_sizes.is_some() {
            return Err("nominal sizedness rules were already sealed");
        }
        for kind in &self.kinds {
            if let Kind::Adt(item, _) = *kind
                && rules.get(item).is_none()
            {
                return Err("nominal sizedness rule is incomplete");
            }
        }
        let changes_properties = rules
            .entries
            .values()
            .any(|entry| entry.rule != Rule::Sized);
        self.nominal_sizes = Some(rules);
        if !changes_properties {
            return Ok(());
        }
        let mut traits = vec![false; self.trait_application_count()];
        for index in 0..self.kinds.len() {
            if let Kind::Projection(application) = self.kinds[index] {
                let id = application.trait_application;
                if !std::mem::replace(&mut traits[id.0 as usize], true) {
                    // All application arguments precede its first projection,
                    // even though applications occupy a separate canonical arena.
                    self.refresh_trait_properties(id)?;
                }
            }
            self.properties[index] = self.properties(self.kinds[index])?;
        }
        for (index, refreshed) in traits.into_iter().enumerate() {
            if !refreshed {
                let index = u32::try_from(index).map_err(|_| "too many trait applications")?;
                self.refresh_trait_properties(super::traits::ApplicationId(index))?;
            }
        }
        Ok(())
    }
}
