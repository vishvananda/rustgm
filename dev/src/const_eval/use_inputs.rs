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
use std::{rc::Rc, sync::Arc};
#[path = "selector_rows.rs"]
mod selectors;
pub(super) use selectors::Opening;

#[derive(Default)]
pub(crate) struct Uses {
    pub needed: bool,
    selectors: Option<Box<selectors::Rows>>,
    #[cfg(test)]
    pub descriptor_work: usize,
}
pub(crate) struct Source {
    contract: Arc<Contract>,
    selectors: Option<Box<selectors::Rows>>,
}
impl Source {
    /// Only the common completed Code-purpose producer calls this constructor.
    pub(crate) fn prepare(
        code: &Code,
        contract: Arc<Contract>,
        uses: Uses,
        bytes: usize,
    ) -> Result<Self, &'static str> {
        if contract.key != code.owner.key(code.source)
            || contract.source != code.source
            || contract.signature != code.owner.signatures().map(|(_, owned)| owned)
        {
            return Err("selector input plan differs from its actual proved code and contract");
        }
        let fixed = size_of::<Row>().saturating_add(contract.retained_bytes());
        let available = bytes
            .checked_sub(fixed)
            .ok_or("source const-call input storage exceeds budget")?;
        if !uses.needed || uses.selectors.as_deref().map_or(0, selectors::Rows::bytes) > available {
            return Err("reference selector data exceeds its retained caller owner");
        }
        Ok(Self {
            contract,
            selectors: uses.selectors,
        })
    }
}

pub(crate) struct Row {
    code: usize,
    executable: Rc<Code>,
    selectors: Option<Box<selectors::Rows>>,
    key: BodyKey,
    source: BodyOrigin,
    instance: Option<Instance>,
    signatures: Option<(TypeId, TypeId)>,
    result: TypeId,
    contract: Arc<Contract>,
    requirements: Option<Arc<Requirements>>,
}
pub(crate) struct PreparedRow {
    index: usize,
    row: Rc<Row>,
    bytes: usize,
}
#[derive(Default)]
pub(crate) struct Inputs {
    rows: HashMap<usize, Rc<Row>>,
    bytes: usize,
}
impl Row {
    pub(super) fn validate(&self, code: usize, body: &Code) -> Result<(), &'static str> {
        if self.code != code
            || !std::ptr::eq(self.executable.as_ref(), body)
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
    pub(crate) fn prepare_uses(
        facts: &mut Declarations,
        body: &Body,
        bytes: usize,
    ) -> Result<Uses, &'static str> {
        if facts.source_bindings.is_none() {
            return Ok(Uses::default());
        }
        if !body.const_context {
            return Ok(Uses::default());
        }
        let mut uses = Uses {
            needed: function_payment(facts, body.owner, body.instance)?,
            ..Uses::default()
        };
        let mut descriptor_work = 0;
        let result: Result<(), &'static str> = (|| {
            for (index, value) in body.values.iter().enumerate() {
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
                    let selection = crate::sema::compiler_items::prepare_select(facts, raw)?
                        .ok_or("source selector preparation lost its actual declaration")?;
                    let call = crate::body::ir::ValueId(
                        u32::try_from(index).map_err(|_| "too many source selector values")?,
                    );
                    selectors::push(
                        facts,
                        &mut uses.selectors,
                        call,
                        selection,
                        bytes,
                        &mut descriptor_work,
                    )?;
                    uses.needed = true;
                    continue;
                }
                if facts.members.constructor(definition).is_none()
                    && facts.is_const_function(definition)
                    && catalog.requires_const_payment(definition)?
                {
                    uses.needed = true;
                }
            }
            Ok(())
        })();
        facts.body_work.canonical_steps = facts
            .body_work
            .canonical_steps
            .saturating_add(descriptor_work);
        #[cfg(test)]
        {
            uses.descriptor_work = descriptor_work;
        }
        result?;
        Ok(uses)
    }
    #[cfg(test)]
    pub(crate) fn prepare(facts: &mut Declarations, body: &Body) -> Result<bool, &'static str> {
        Self::prepare_uses(facts, body, 64 * 1024 * 1024).map(|uses| uses.needed)
    }
    pub(crate) fn plan(
        &mut self,
        facts: &mut Declarations,
        index: usize,
        executable: Rc<Code>,
        source: Source,
        selected: Option<Instance>,
    ) -> Result<PreparedRow, &'static str> {
        let Source {
            contract,
            selectors,
        } = source;
        let body = executable.as_ref();
        if self.rows.contains_key(&index) {
            return Err("source const-call input owner was published twice");
        }
        if contract.key != body.owner.key(body.source)
            || contract.source != body.source
            || contract.signature != body.owner.signatures().map(|(_, owned)| owned)
        {
            return Err("source const-call contract belongs to another body");
        }
        let base = contract
            .retained_bytes()
            .saturating_add(size_of::<Row>())
            .saturating_add(selectors.as_deref().map_or(0, selectors::Rows::bytes));
        let growth = if self.rows.len() < self.rows.capacity() {
            0
        } else {
            crate::support::retained::new_map::<usize, Rc<Row>>(self.rows.len() + 1)
        };
        let fixed = self
            .retained_bytes()
            .saturating_add(base)
            .saturating_add(growth);
        let budget = (64usize * 1024 * 1024)
            .checked_sub(fixed)
            .ok_or("source const-call input storage exceeds budget")?;
        let requirements = if function_payment(facts, body.owner, body.instance)? {
            Some(const_payments::prepare(
                facts,
                selected
                    .or(body.instance)
                    .ok_or("conditional code needs its complete selected instance")?,
                budget,
            )?)
        } else {
            None
        };
        let retained = base.saturating_add(requirements.as_ref().map_or(0, |r| r.retained_bytes()));
        let total = self
            .retained_bytes()
            .saturating_add(retained)
            .saturating_add(growth);
        if total > 64 * 1024 * 1024 {
            return Err("source const-call input storage exceeds budget");
        }
        let row = Rc::new(Row {
            code: index,
            selectors,
            key: body.owner.key(body.source),
            source: body.source,
            instance: body.instance,
            signatures: body.owner.signatures(),
            result: body.result,
            contract,
            requirements,
            executable,
        });
        row.validate(index, &row.executable)?;
        self.rows
            .try_reserve(1)
            .map_err(|_| "source input index allocation failed")?;
        if self.retained_bytes().saturating_add(retained) > 64 * 1024 * 1024 {
            return Err("source input index payload exceeds storage budget");
        }
        Ok(PreparedRow {
            index,
            row,
            bytes: retained,
        })
    }
    pub(crate) fn retained_bytes(&self) -> usize {
        size_of::<Self>()
            .saturating_add(self.bytes)
            .saturating_add(crate::support::retained::map(&self.rows))
    }
    pub(crate) fn remaining_bytes(&self) -> usize {
        (64 * 1024 * 1024usize).saturating_sub(self.retained_bytes())
    }
    pub(in crate::const_eval) fn check_plan(
        &self,
        plan: &PreparedRow,
        index: usize,
        code: &Rc<Code>,
    ) -> Result<(), &'static str> {
        if plan.index != index
            || !Rc::ptr_eq(&plan.row.executable, code)
            || self.rows.contains_key(&index)
            || self.rows.len() >= self.rows.capacity()
            || self.retained_bytes().saturating_add(plan.bytes) > 64 * 1024 * 1024
        {
            return Err("source input plan changed its exact code, index or storage owner");
        }
        plan.row.validate(index, code)
    }
    pub(in crate::const_eval) fn commit(&mut self, plan: PreparedRow) {
        self.bytes = self.bytes.saturating_add(plan.bytes);
        self.rows.insert(plan.index, plan.row);
    }
    pub(super) fn opening(
        &self,
        catalog: &Catalog,
        code: usize,
        body: &Code,
        call: crate::body::ir::ValueId,
    ) -> Result<Opening, &'static str> {
        let row = self.checked(catalog, code, body)?;
        row.selectors
            .as_deref()
            .and_then(|rows| rows.get(call))
            .ok_or("reference selector has no proved input/output descriptor")
    }
    pub(super) fn checked(
        &self,
        catalog: &Catalog,
        index: usize,
        code: &Code,
    ) -> Result<&Rc<Row>, &'static str> {
        catalog.validate_entry(index)?;
        if catalog
            .code(index)
            .is_none_or(|actual| !std::ptr::eq(actual, code))
        {
            return Err("source inputs belong to another actual catalog code");
        }
        let row = self
            .rows
            .get(&index)
            .ok_or("source caller has no actual retained row")?;
        row.validate(index, code)?;
        Ok(row)
    }
    pub(super) fn get(&self, code: usize, catalog: &Catalog) -> Result<Rc<Row>, &'static str> {
        let body = catalog
            .code(code)
            .ok_or("source conditional code disappeared")?;
        self.checked(catalog, code, body).map(Rc::clone)
    }
}

fn function_payment(
    facts: &mut Declarations,
    owner: Owner,
    instance: Option<Instance>,
) -> Result<bool, &'static str> {
    let Owner::Function { definition, .. } = owner else {
        return Ok(false);
    };
    facts.charge_caller_inputs(8, 0)?;
    // A default's implicit Self contract is proved at its reached call. It is
    // not the free-function conditional effect represented by Requirements.
    if let Some(instance) = instance
        && facts.const_default_member(instance)? == Some(definition)
    {
        return Ok(false);
    }
    facts
        .source_bindings
        .as_deref()
        .ok_or("source const catalog disappeared")?
        .requires_const_payment(definition)
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
