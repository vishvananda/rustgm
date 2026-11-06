//! Actual nominal adjustment uses travel with the existing callable input owner.
use super::{Body, CallableInputs, NodeKey, Rvalue, TypeId, ValueId, retained};
use crate::{
    body::ir::{BodyKey, BodyOrigin},
    sema::model::Declarations,
    types::{arena::Identity, instance::Instance, substitute::Bound, unsizing::Conversion},
};
#[cfg(test)]
#[path = "../../tests/invariants/transfer_claims.rs"]
mod tests;

#[derive(Clone, Copy)]
pub(crate) struct NominalUse {
    pub source: NodeKey,
    pub operation: ValueId,
    pub operand: ValueId,
    pub from: TypeId,
    pub to: TypeId,
}
#[derive(Clone)]
pub(super) struct NominalInputs {
    key: BodyKey,
    source: BodyOrigin,
    instance: Option<Instance>,
    arena: Identity,
    uses: Vec<NominalUse>,
}
impl NominalInputs {
    pub(super) fn allocation_bytes(&self) -> usize {
        size_of::<Self>().saturating_add(retained::vector(&self.uses))
    }
    fn require(&self, types: &crate::types::Store, body: &Body) -> Result<(), &'static str> {
        if self.key != body.key()
            || self.source != body.source
            || self.instance != body.instance
            || !types.has_identity(&self.arena)
        {
            return Err("nominal adjustment belongs to another body/instance/type store");
        }
        Ok(())
    }
}
impl CallableInputs {
    pub(in crate::body) fn nominal_uses(&self) -> &[NominalUse] {
        self.nominal
            .as_ref()
            .map_or(&[], |inputs| inputs.uses.as_slice())
    }
}
impl Body {
    pub(crate) fn nominal_use(
        &self,
        types: &crate::types::Store,
        operation: ValueId,
    ) -> Result<NominalUse, &'static str> {
        let inputs = self
            .resolved_callables
            .as_ref()
            .and_then(|inputs| inputs.nominal.as_ref())
            .ok_or("nominal conversion lacks its actual adjustment provenance")?;
        inputs.require(types, self)?;
        let row = inputs
            .uses
            .binary_search_by_key(&operation.0, |row| row.operation.0)
            .ok()
            .and_then(|index| inputs.uses.get(index))
            .copied()
            .ok_or("nominal conversion lacks its actual operation provenance")?;
        self.validate_nominal_operand(row)?;
        Ok(row)
    }
    pub(in crate::body) fn nominal_uses(&self) -> &[NominalUse] {
        self.resolved_callables
            .as_deref()
            .map_or(&[], CallableInputs::nominal_uses)
    }
    pub(super) fn copy_nominal_inputs(
        &self,
        facts: &Declarations,
        key: BodyKey,
        instance: Option<Instance>,
    ) -> Result<Option<Box<NominalInputs>>, &'static str> {
        let Some(inputs) = self
            .resolved_callables
            .as_ref()
            .and_then(|inputs| inputs.nominal.as_ref())
        else {
            return Ok(None);
        };
        self.validate_nominal_inputs(facts)?;
        if key != self.key() {
            return Err("nominal adjustment requires its actual selected owner transport");
        }
        facts.charge_caller_inputs(
            inputs.uses.len(),
            size_of::<NominalInputs>()
                .saturating_add(std::mem::size_of_val(inputs.uses.as_slice())),
        )?;
        Ok(Some(Box::new(NominalInputs {
            key,
            source: inputs.source,
            instance,
            arena: inputs.arena.clone(),
            uses: inputs.uses.clone(),
        })))
    }
    pub(in crate::body) fn append_nominal_use(
        &mut self,
        facts: &Declarations,
        row: NominalUse,
    ) -> Result<(), &'static str> {
        let Some(Conversion::Nominal { source, target }) =
            facts.unsizing_adjustments.get(&row.source)
        else {
            return Err("nominal input requires its actual checked source adjustment");
        };
        if (*source, *target) != (row.from, row.to) {
            return Err("nominal input changed its checked adjustment endpoints");
        }
        let previous = self.resolved_callables.as_deref();
        let nominal = previous.and_then(|inputs| inputs.nominal.as_deref());
        if let Some(inputs) = nominal {
            inputs.require(&facts.types, self)?;
        }
        let (length, capacity) =
            nominal.map_or((0, 0), |inputs| (inputs.uses.len(), inputs.uses.capacity()));
        if length >= 1_000_000 {
            return Err("nominal adjustment inventory exhausted");
        }
        let arena = facts
            .types
            .existing_arena_identity()
            .ok_or("nominal adjustment lacks its prepaid type-store identity")?;
        let growth = retained::Growth::vector::<NominalUse>(length, capacity, 1);
        facts.charge_caller_inputs(
            growth.work.saturating_add(1),
            growth
                .bytes
                .saturating_add(if nominal.is_none() {
                    size_of::<NominalInputs>()
                } else {
                    0
                })
                .saturating_add(if previous.is_none() {
                    size_of::<CallableInputs>()
                } else {
                    0
                }),
        )?;
        self.validate_nominal_use(facts, row)?;
        let key = self.key();
        let source = self.source;
        let instance = self.instance;
        let inputs = self.resolved_callables.get_or_insert_default();
        let nominal = inputs.nominal.get_or_insert_with(|| {
            Box::new(NominalInputs {
                key,
                source,
                instance,
                arena: arena.clone(),
                uses: Vec::new(),
            })
        });
        if nominal
            .uses
            .last()
            .is_some_and(|last| last.operation.0 >= row.operation.0)
        {
            return Err("nominal adjustment operation is duplicate or unordered");
        }
        nominal
            .uses
            .try_reserve(1)
            .map_err(|_| "nominal adjustment allocation failed")?;
        nominal.uses.push(row);
        Ok(())
    }
    fn validate_nominal_use(
        &self,
        facts: &Declarations,
        row: NominalUse,
    ) -> Result<(), &'static str> {
        let Some(Conversion::Nominal { .. }) = facts.unsizing_adjustments.get(&row.source) else {
            return Err("nominal adjustment lost its actual source operation");
        };
        self.validate_nominal_operand(row)
    }
    fn validate_nominal_operand(&self, row: NominalUse) -> Result<(), &'static str> {
        let value = self
            .values
            .get(row.operation.0 as usize)
            .ok_or("nominal adjustment lost its actual operation")?;
        if self
            .source
            .source()
            .is_none_or(|source| source.file != row.source.file)
            || row.operand.0 >= row.operation.0
            || self
                .values
                .get(row.operand.0 as usize)
                .is_none_or(|v| v.ty != row.from)
            || value.ty != row.to
            || !matches!(value.kind,
                Rvalue::Unsize { value, conversion: Conversion::Nominal { source, target } }
                if value == row.operand && source == row.from && target == row.to)
        {
            return Err("nominal adjustment changed its actual source/operand/endpoints");
        }
        Ok(())
    }
    pub(in crate::body) fn validate_nominal_inputs(
        &self,
        facts: &Declarations,
    ) -> Result<(), &'static str> {
        let Some(inputs) = self
            .resolved_callables
            .as_ref()
            .and_then(|inputs| inputs.nominal.as_ref())
        else {
            return Ok(());
        };
        inputs.require(&facts.types, self)?;
        facts.charge_caller_inputs(inputs.uses.len().saturating_add(self.values.len()), 0)?;
        if inputs.uses.is_empty()
            || self
                .values
                .iter()
                .filter(|value| {
                    matches!(
                        value.kind,
                        Rvalue::Unsize {
                            conversion: Conversion::Nominal { .. },
                            ..
                        }
                    )
                })
                .count()
                != inputs.uses.len()
        {
            return Err("nominal adjustment inventory does not cover its actual operations");
        }
        let mut previous = None;
        for &row in &inputs.uses {
            if previous.is_some_and(|previous| previous >= row.operation.0) {
                return Err("nominal adjustment operation is duplicate or unordered");
            }
            self.validate_nominal_use(facts, row)?;
            previous = Some(row.operation.0);
        }
        self.validate_nominal_sites(facts, &inputs.uses)
    }
    fn validate_nominal_sites(
        &self,
        facts: &Declarations,
        rows: &[NominalUse],
    ) -> Result<(), &'static str> {
        let lookup = rows.len().ilog2() as usize + 2;
        facts.charge_caller_inputs(self.statements.len().saturating_mul(lookup), 0)?;
        let mut cursor = 0;
        // Builder operations are source ordered. The one existing row column
        // authenticates their actual Evaluate sites without a second index.
        for statement in &self.statements {
            let crate::body::ir::Action::Evaluate(operation) = statement.action else {
                continue;
            };
            let Ok(index) = rows.binary_search_by_key(&operation.0, |row| row.operation.0) else {
                continue;
            };
            if index != cursor || statement.source != rows[index].source.node.into() {
                return Err("nominal adjustment changed its actual Evaluate/source identity");
            }
            cursor += 1;
        }
        if cursor != rows.len() {
            return Err("nominal adjustment lost its actual Evaluate operation");
        }
        Ok(())
    }
    pub(in crate::body) fn rewrite_nominal_inputs(
        &mut self,
        facts: &mut Declarations,
        fold: &mut Bound,
    ) -> Result<(), &'static str> {
        if let Some(inputs) = self
            .resolved_callables
            .as_ref()
            .and_then(|inputs| inputs.nominal.as_ref())
            && !facts.types.has_identity(&inputs.arena)
        {
            return Err("nominal adjustment belongs to another type store");
        }
        self.map_nominal_inputs(|ty| {
            let before = fold.steps;
            let result = fold.apply(&mut facts.types, ty);
            facts.charge_caller_inputs(fold.steps.saturating_sub(before), 0)?;
            result
        })
    }
    pub(in crate::body) fn map_nominal_inputs(
        &mut self,
        mut map: impl FnMut(TypeId) -> Result<TypeId, &'static str>,
    ) -> Result<(), &'static str> {
        let Some(inputs) = self
            .resolved_callables
            .as_deref_mut()
            .and_then(|inputs| inputs.nominal.as_deref_mut())
        else {
            return Ok(());
        };
        for row in &mut inputs.uses {
            row.from = map(row.from)?;
            row.to = map(row.to)?;
        }
        Ok(())
    }
}

const _: () = {
    assert!(size_of::<NominalUse>() == 24);
    assert!(size_of::<NominalInputs>() == 72);
    assert!(size_of::<CallableInputs>() == 64);
};
