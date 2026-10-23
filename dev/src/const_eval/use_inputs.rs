//! Checked source code retains unmet contracts, never another caller's permission.
use super::code::{Catalog, Code};
use crate::{
    body::{
        ir::{Body, BodyKey, BodyOrigin, Owner, Rvalue},
        regions::Contract,
    },
    sema::{
        compiler_items::const_payments::{self, Requirements},
        model::Declarations,
    },
    support::hash::HashMap,
    types::{Kind, TypeId, instance::Instance, parameters::environment::Environment},
};
use std::sync::Arc;

pub(crate) struct Row {
    code: usize,
    key: BodyKey,
    source: BodyOrigin,
    instance: Option<Instance>,
    signatures: Option<(TypeId, TypeId)>,
    result: TypeId,
    contract: Arc<Contract>,
    requirements: Option<Arc<Requirements>>,
}
#[derive(Default)]
pub(crate) struct Inputs {
    rows: HashMap<usize, Arc<Row>>,
    bytes: usize,
}
impl Row {
    pub(super) fn validate(&self, code: usize, body: &Code) -> Result<(), &'static str> {
        if self.code != code
            || self.key != body.owner.key(body.source)
            || self.source != body.source
            || self.instance != body.instance
            || self.signatures != body.owner.signatures()
            || self.result != body.result
            || !body.const_context
            || self.contract.key != self.key
            || self.contract.source != self.source
            || self.contract.signature != self.signatures.map(|(_, owned)| owned)
        {
            return Err("source const-call inputs belong to another checked code or contract");
        }
        if let Some(requirements) = &self.requirements
            && (self.signatures.map(|(public, _)| public) != Some(requirements.signature())
                || self.result != crate::types::TypeId::primitive(requirements.result()))
        {
            return Err("source const callee changed its paired callable signature");
        }
        Ok(())
    }
    pub(super) fn environment(&self) -> Option<&Arc<Environment>> {
        self.contract
            .builtins
            .as_deref()
            .and_then(|b| b.environment.as_ref())
    }
    pub(super) fn requirements(&self) -> Result<&Arc<Requirements>, &'static str> {
        self.requirements
            .as_ref()
            .ok_or("source const callee has no materialized effect requirements")
    }
    pub(super) fn contract(&self) -> &Arc<Contract> {
        &self.contract
    }
}
impl Inputs {
    /// Canonical relocation preparation is data, never callback body demand.
    /// Complete every selector even when an earlier use already needs its row.
    pub(crate) fn prepare(facts: &mut Declarations, body: &Body) -> Result<bool, &'static str> {
        let Some(catalog) = facts.source_bindings.as_deref() else {
            return Ok(false);
        };
        if !body.const_context {
            return Ok(false);
        }
        let mut needed = if let Owner::Function { definition, .. } = body.owner {
            catalog.requires_const_payment(definition)?
        } else {
            false
        };
        for value in &body.values {
            facts.body_work.canonical_steps = facts.body_work.canonical_steps.saturating_add(1);
            let Rvalue::Call { callee, .. } = value.kind else {
                continue;
            };
            let Some(ty) = body.values.get(callee.0 as usize).map(|v| v.ty) else {
                continue;
            };
            let Some(Kind::FunctionItem { item, .. }) = facts.types.kind(ty) else {
                continue;
            };
            let definition = crate::resolve::model::DefId(item.0);
            let catalog = facts
                .source_bindings
                .as_deref()
                .ok_or("source catalog disappeared")?;
            if catalog.intrinsic_kind(definition)?
                == Some(crate::support::bindings::SourceIntrinsic::ConstEvalSelect)
            {
                let raw = Instance::new(&facts.types, ty)?;
                crate::sema::compiler_items::prepare_select(facts, raw)?
                    .ok_or("source selector preparation lost its actual declaration")?;
                needed = true;
                continue;
            }
            if facts.members.constructor(definition).is_none()
                && facts.is_const_function(definition)
                && catalog.requires_const_payment(definition)?
            {
                needed = true;
            }
        }
        Ok(needed)
    }
    pub(crate) fn record(
        &mut self,
        facts: &mut Declarations,
        catalog: &Catalog,
        index: usize,
        body: &Body,
        contract: Arc<Contract>,
        selected: Option<Instance>,
    ) -> Result<(), &'static str> {
        if self.rows.contains_key(&index) {
            return Err("source const-call input owner was published twice");
        }
        if contract.key != body.key()
            || contract.source != body.source
            || contract.signature != body.owner.signatures().map(|(_, owned)| owned)
        {
            return Err("source const-call contract belongs to another body");
        }
        let base = contract.retained_bytes().saturating_add(size_of::<Row>());
        let fixed = size_of::<Self>()
            .saturating_add(self.bytes)
            .saturating_add(base)
            .saturating_add(crate::support::retained::new_map::<usize, Arc<Row>>(
                self.rows.len() + 1,
            ));
        let budget = (64usize * 1024 * 1024)
            .checked_sub(fixed)
            .ok_or("source const-call input storage exceeds budget")?;
        let requirements = if let Owner::Function { definition, .. } = body.owner {
            if facts
                .source_bindings
                .as_deref()
                .ok_or("source const catalog disappeared")?
                .requires_const_payment(definition)?
            {
                Some(const_payments::prepare(
                    facts,
                    selected
                        .or(body.instance)
                        .ok_or("conditional code needs its complete selected instance")?,
                    budget,
                )?)
            } else {
                None
            }
        } else {
            None
        };
        let retained = base.saturating_add(requirements.as_ref().map_or(0, |r| r.retained_bytes()));
        let total = size_of::<Self>()
            .saturating_add(self.bytes)
            .saturating_add(retained)
            .saturating_add(crate::support::retained::new_map::<usize, Arc<Row>>(
                self.rows.len() + 1,
            ));
        if total > 64 * 1024 * 1024 {
            return Err("source const-call input storage exceeds budget");
        }
        let row = Arc::new(Row {
            code: index,
            key: body.key(),
            source: body.source,
            instance: body.instance,
            signatures: body.owner.signatures(),
            result: body.result,
            contract,
            requirements,
        });
        row.validate(
            index,
            catalog
                .code(index)
                .ok_or("source code publication disappeared")?,
        )?;
        self.rows.insert(index, row);
        self.bytes = self.bytes.saturating_add(retained);
        Ok(())
    }
    pub(crate) fn retained_bytes(&self) -> usize {
        size_of::<Self>().saturating_add(self.bytes).saturating_add(
            crate::support::retained::new_map::<usize, Arc<Row>>(self.rows.len()),
        )
    }
    pub(super) fn get(&self, code: usize, catalog: &Catalog) -> Result<Arc<Row>, &'static str> {
        catalog.validate_entry(code)?;
        let row = self
            .rows
            .get(&code)
            .ok_or("source conditional use has no checked materialized caller/callee inputs")?;
        row.validate(
            code,
            catalog
                .code(code)
                .ok_or("source conditional code disappeared")?,
        )?;
        Ok(Arc::clone(row))
    }
}

#[cfg(test)]
impl Inputs {
    pub(super) fn observations(&self) -> (usize, usize, usize) {
        (
            self.rows.len(),
            self.rows
                .values()
                .filter(|r| r.requirements.is_some())
                .count(),
            self.retained_bytes(),
        )
    }
}
