//! Sparse caller-site data shares the existing callable input owner, never its proof.
use super::ir::{Body, NodeKey, Rvalue, ValueId};
use crate::{
    sema::values::LocationId,
    support::{hash::HashMap, retained},
    types::{TypeId, instance::Instance},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct CallerSite {
    pub owner: NodeKey,
    pub value: ValueId,
    pub callable: TypeId,
    pub location: LocationId,
}
#[derive(Clone, Default)]
pub struct CallableInputs {
    pub(super) targets: Option<HashMap<Instance, Instance>>,
    pub(super) sites: Vec<CallerSite>,
}
impl CallableInputs {
    pub(crate) fn from_targets(targets: HashMap<Instance, Instance>) -> Self {
        Self {
            targets: Some(targets),
            sites: Vec::new(),
        }
    }
    pub(super) fn allocation_bytes(&self) -> usize {
        self.targets
            .as_ref()
            .map_or(0, retained::map)
            .saturating_add(retained::vector(&self.sites))
    }
}
impl Body {
    pub(crate) fn has_resolved_callables(&self) -> bool {
        self.resolved_callables
            .as_ref()
            .is_some_and(|inputs| inputs.targets.is_some())
    }
    pub(crate) fn install_callable_targets(
        &mut self,
        targets: HashMap<Instance, Instance>,
    ) -> Result<(), &'static str> {
        if self.has_resolved_callables() {
            return Err("callable targets already belong to a completed proof");
        }
        if let Some(inputs) = self.resolved_callables.as_deref_mut() {
            inputs.targets = Some(targets);
        } else {
            self.resolved_callables = Some(Box::new(CallableInputs::from_targets(targets)));
        }
        Ok(())
    }
    pub(crate) fn caller_sites(&self) -> &[CallerSite] {
        self.resolved_callables
            .as_ref()
            .map_or(&[], |inputs| inputs.sites.as_slice())
    }
    pub(crate) fn caller_site(&self, value: ValueId) -> Result<Option<CallerSite>, &'static str> {
        let sites = self.caller_sites();
        let site = sites
            .binary_search_by_key(&value.0, |site| site.value.0)
            .ok()
            .map(|index| sites[index]);
        if site.is_some_and(|site| Some(site.owner) != self.source.source()) {
            return Err("caller location belongs to another actual body");
        }
        Ok(site)
    }
    pub(crate) fn append_caller_site(
        &mut self,
        site: CallerSite,
        facts: &crate::sema::model::Declarations,
    ) -> Result<(), &'static str> {
        let previous = self.resolved_callables.as_deref();
        let (length, capacity) = previous.map_or((0, 0), |inputs| {
            (inputs.sites.len(), inputs.sites.capacity())
        });
        if length >= 1_000_000 {
            return Err("caller-site inventory exhausted");
        }
        let growth = retained::Growth::vector::<CallerSite>(length, capacity, 1);
        facts.charge_caller_inputs(
            growth.work.saturating_add(1),
            growth.bytes.saturating_add(if previous.is_none() {
                size_of::<CallableInputs>()
            } else {
                0
            }),
        )?;
        let sites = &mut self.resolved_callables.get_or_insert_default().sites;
        sites
            .try_reserve(1)
            .map_err(|_| "caller-site allocation failed")?;
        sites.push(site);
        Ok(())
    }
    pub(crate) fn seal_caller_sites(
        &mut self,
        facts: &crate::sema::model::Declarations,
    ) -> Result<(), &'static str> {
        let Some(inputs) = self
            .resolved_callables
            .as_mut()
            .filter(|inputs| !inputs.sites.is_empty())
        else {
            return Ok(());
        };
        let count = inputs.sites.len();
        facts.charge_caller_inputs(count.saturating_mul(count.ilog2() as usize + 2), 0)?;
        inputs.sites.sort_unstable_by_key(|site| site.value.0);
        self.validate_caller_sites()
    }
    pub(crate) fn validate_caller_sites(&self) -> Result<(), &'static str> {
        let mut previous = None;
        for site in self.caller_sites() {
            let value = self
                .values
                .get(site.value.0 as usize)
                .ok_or("caller-site value is missing")?;
            let Rvalue::Call { callee, .. } = value.kind else {
                return Err("caller site requires its actual Call");
            };
            if previous.is_some_and(|previous| previous >= site.value.0)
                || Some(site.owner) != self.source.source()
                || self
                    .values
                    .get(callee.0 as usize)
                    .is_none_or(|callee| callee.ty != site.callable)
            {
                return Err("caller-site body/value/callee identity changed");
            }
            previous = Some(site.value.0);
        }
        Ok(())
    }
    pub(crate) fn unproved_callable_inputs(
        &self,
        facts: &crate::sema::model::Declarations,
    ) -> Result<Option<Box<CallableInputs>>, &'static str> {
        let Some(inputs) = self
            .resolved_callables
            .as_ref()
            .filter(|inputs| !inputs.sites.is_empty())
        else {
            return Ok(None);
        };
        facts.charge_caller_inputs(
            inputs.sites.len(),
            size_of::<CallableInputs>()
                .saturating_add(std::mem::size_of_val(inputs.sites.as_slice())),
        )?;
        Ok(Some(Box::new(CallableInputs {
            targets: None,
            sites: inputs.sites.clone(),
        })))
    }
    pub(crate) fn specialize_caller_inputs(
        &mut self,
        facts: &crate::sema::model::Declarations,
    ) -> Result<(), &'static str> {
        let Some(inputs) = self
            .resolved_callables
            .as_deref_mut()
            .filter(|inputs| !inputs.sites.is_empty())
        else {
            return Ok(());
        };
        facts.charge_caller_inputs(inputs.sites.len(), 0)?;
        for site in &mut inputs.sites {
            let Rvalue::Call { callee, .. } = self
                .values
                .get(site.value.0 as usize)
                .ok_or("specialized caller site lost its value")?
                .kind
            else {
                return Err("specialized caller site changed its Call");
            };
            site.callable = self
                .values
                .get(callee.0 as usize)
                .ok_or("specialized caller site lost its callee")?
                .ty;
        }
        Ok(())
    }
    /// Dispatch transports the sealed callable through the same proved type map
    /// as its operand. Source owner, value and immutable location stay exact.
    pub(in crate::body) fn rewrite_caller_inputs(
        &mut self,
        facts: &mut crate::sema::model::Declarations,
        fold: &mut crate::types::substitute::Bound,
    ) -> Result<(), &'static str> {
        let Some(inputs) = self
            .resolved_callables
            .as_deref_mut()
            .filter(|inputs| !inputs.sites.is_empty())
        else {
            return Ok(());
        };
        facts.charge_caller_inputs(inputs.sites.len(), 0)?;
        for site in &mut inputs.sites {
            let before = fold.steps;
            let mapped = fold.apply(&mut facts.types, site.callable);
            facts.charge_caller_inputs(fold.steps.saturating_sub(before), 0)?;
            site.callable = mapped?;
        }
        self.validate_caller_sites()
    }
}
