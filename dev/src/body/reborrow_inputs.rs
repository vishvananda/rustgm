//! Opaque value-borrow uses bind checked source facts to one actual CFG operation.
use crate::{
    body::ir::{Body, BodyKey, BodyOrigin, Place, Rvalue, ValueId},
    resolve::unit::NodeKey,
    sema::{model::Declarations, reborrow_adjustments::Field},
    support::retained::Growth,
    types::{Store, TypeId, arena::Identity, instance::Instance},
};

#[derive(Clone, Copy)]
pub(crate) struct ValueReborrowUse {
    pub site: NodeKey,
    pub operation: ValueId,
    pub place: Place,
    pub source: TypeId,
    pub target: TypeId,
    pub mutable: bool,
    pub field: Option<Field>,
}
#[derive(Clone)]
pub(crate) struct Inputs {
    key: BodyKey,
    origin: BodyOrigin,
    instance: Option<Instance>,
    arena: Identity,
    uses: Vec<ValueReborrowUse>,
}
impl Inputs {
    pub(crate) fn require_context(
        &self,
        key: BodyKey,
        origin: BodyOrigin,
        instance: Option<Instance>,
    ) -> Result<(), &'static str> {
        if self.key != key || self.origin != origin || self.instance != instance {
            return Err("value reborrow belongs to another body/source/instance");
        }
        Ok(())
    }
    pub(crate) fn require_store(&self, types: &Store) -> Result<(), &'static str> {
        if !types.has_identity(&self.arena) {
            return Err("value reborrow belongs to another type store");
        }
        Ok(())
    }
    pub(crate) fn selected(
        &self,
        types: &Store,
        key: BodyKey,
        origin: BodyOrigin,
        instance: Option<Instance>,
        operation: ValueId,
    ) -> Result<ValueReborrowUse, &'static str> {
        self.require_context(key, origin, instance)?;
        self.require_store(types)?;
        self.uses
            .binary_search_by_key(&operation.0, |r| r.operation.0)
            .ok()
            .and_then(|i| self.uses.get(i))
            .copied()
            .ok_or("value reborrow operation was not captured")
    }
    pub(crate) fn map_types(
        &mut self,
        map: &mut impl FnMut(TypeId) -> Result<TypeId, &'static str>,
    ) -> Result<(), &'static str> {
        for row in &mut self.uses {
            row.source = map(row.source)?;
            row.target = map(row.target)?;
            if let Some(field) = &mut row.field {
                field.source_ty = map(field.source_ty)?;
                field.target_ty = map(field.target_ty)?;
            }
        }
        Ok(())
    }
    pub(crate) fn allocation_bytes(&self) -> usize {
        size_of::<Self>().saturating_add(crate::support::retained::vector(&self.uses))
    }
    fn require(&self, types: &Store, body: &Body) -> Result<(), &'static str> {
        self.require_context(body.key(), body.source, body.instance)?;
        if !types.has_identity(&self.arena) {
            return Err("value reborrow belongs to another body/source/instance/type store");
        }
        Ok(())
    }
}
impl Body {
    pub(in crate::body) fn require_value_reborrow_context(
        &self,
        types: &Store,
    ) -> Result<(), &'static str> {
        if let Some(inputs) = self
            .resolved_callables
            .as_deref()
            .and_then(|p| p.value_reborrows.as_deref())
        {
            inputs.require(types, self)?;
        }
        Ok(())
    }
    pub(crate) fn reborrow_snapshot_bytes(&self) -> usize {
        self.resolved_callables
            .as_deref()
            .and_then(|p| p.value_reborrows.as_deref())
            .map_or(0, |p| {
                p.allocation_bytes().saturating_add(2 * size_of::<usize>())
            })
    }
    pub(crate) fn snapshot_value_reborrows(
        &self,
    ) -> Result<Option<std::sync::Arc<Inputs>>, &'static str> {
        let Some(inputs) = self
            .resolved_callables
            .as_deref()
            .and_then(|p| p.value_reborrows.as_deref())
        else {
            return Ok(None);
        };
        inputs.require_context(self.key(), self.source, self.instance)?;
        for row in &inputs.uses {
            self.validate_value_reborrow_operand(*row)?;
        }
        Ok(Some(std::sync::Arc::new(inputs.clone())))
    }
    pub(crate) fn value_reborrow(
        &self,
        types: &Store,
        operation: ValueId,
    ) -> Result<ValueReborrowUse, &'static str> {
        let inputs = self
            .resolved_callables
            .as_deref()
            .and_then(|p| p.value_reborrows.as_deref())
            .ok_or("value reborrow lacks its actual source operation provenance")?;
        inputs.require(types, self)?;
        let row = inputs
            .uses
            .binary_search_by_key(&operation.0, |r| r.operation.0)
            .ok()
            .and_then(|i| inputs.uses.get(i))
            .copied()
            .ok_or("value reborrow operation was not captured")?;
        self.validate_value_reborrow_operand(row)?;
        Ok(row)
    }
    fn validate_value_reborrow_operand(&self, row: ValueReborrowUse) -> Result<(), &'static str> {
        let value = self
            .values
            .get(row.operation.0 as usize)
            .ok_or("value reborrow operation disappeared")?;
        if self.place_type(row.place) != Some(row.source)
            || value.ty != row.target
            || !matches!(value.kind,Rvalue::Reborrow{place,mutable}if place==row.place&&mutable==row.mutable)
            || self.source.source().is_none_or(|s| s.file != row.site.file)
        {
            return Err("value reborrow changed its actual operand or endpoints");
        }
        Ok(())
    }
    pub(in crate::body) fn append_value_reborrow(
        &mut self,
        facts: &Declarations,
        site: NodeKey,
        operation: ValueId,
        place: Place,
    ) -> Result<(), &'static str> {
        let adjustment = facts
            .value_reborrows
            .as_deref()
            .ok_or("value reborrow source selection disappeared")?
            .get(facts, site)?
            .ok_or("value reborrow has no checked source selection")?;
        if self.source.source() != Some(adjustment.body) {
            return Err("value reborrow source selection belongs to another body");
        }
        let row = ValueReborrowUse {
            site,
            operation,
            place,
            source: adjustment.source,
            target: adjustment.borrow,
            mutable: adjustment.mutable,
            field: adjustment.field,
        };
        self.validate_value_reborrow_operand(row)?;
        let previous = self.resolved_callables.as_deref();
        let inputs = previous.and_then(|p| p.value_reborrows.as_deref());
        if let Some(inputs) = inputs {
            inputs.require(&facts.types, self)?;
        }
        let (length, capacity) = inputs.map_or((0, 0), |p| (p.uses.len(), p.uses.capacity()));
        let growth = Growth::vector::<ValueReborrowUse>(length, capacity, 1);
        facts.charge_caller_inputs(
            growth.work.saturating_add(1),
            growth
                .bytes
                .saturating_add(if inputs.is_none() {
                    size_of::<Inputs>()
                } else {
                    0
                })
                .saturating_add(if previous.is_none() {
                    size_of::<super::callable_inputs::CallableInputs>()
                } else {
                    0
                }),
        )?;
        let arena = facts
            .types
            .existing_arena_identity()
            .ok_or("value reborrow has no prepaid type-store identity")?;
        let (key, origin, instance) = (self.key(), self.source, self.instance);
        let inputs = self
            .resolved_callables
            .get_or_insert_default()
            .value_reborrows
            .get_or_insert_with(|| {
                Box::new(Inputs {
                    key,
                    origin,
                    instance,
                    arena: arena.clone(),
                    uses: Vec::new(),
                })
            });
        if inputs
            .uses
            .last()
            .is_some_and(|r| r.operation.0 >= operation.0)
        {
            return Err("value reborrow operation is duplicate or unordered");
        }
        inputs
            .uses
            .try_reserve(1)
            .map_err(|_| "value reborrow input allocation failed")?;
        inputs.uses.push(row);
        Ok(())
    }
    pub(super) fn copy_value_reborrow_inputs(
        &self,
        facts: &Declarations,
        key: BodyKey,
        instance: Option<Instance>,
    ) -> Result<Option<Box<Inputs>>, &'static str> {
        let Some(inputs) = self
            .resolved_callables
            .as_deref()
            .and_then(|p| p.value_reborrows.as_deref())
        else {
            return Ok(None);
        };
        self.validate_value_reborrow_inputs(facts)?;
        if key != self.key() {
            return Err("value reborrow requires its actual selected owner transport");
        }
        facts.charge_caller_inputs(inputs.uses.len(), inputs.allocation_bytes())?;
        Ok(Some(Box::new(Inputs {
            key,
            origin: inputs.origin,
            instance,
            arena: inputs.arena.clone(),
            uses: inputs.uses.clone(),
        })))
    }
    pub(in crate::body) fn validate_value_reborrow_inputs(
        &self,
        facts: &Declarations,
    ) -> Result<(), &'static str> {
        let Some(inputs) = self
            .resolved_callables
            .as_deref()
            .and_then(|p| p.value_reborrows.as_deref())
        else {
            return Ok(());
        };
        inputs.require(&facts.types, self)?;
        facts.charge_caller_inputs(self.values.len().saturating_add(inputs.uses.len()), 0)?;
        if self
            .values
            .iter()
            .filter(|v| matches!(v.kind, Rvalue::Reborrow { .. }))
            .count()
            != inputs.uses.len()
        {
            return Err("value reborrow inputs do not cover actual operations");
        }
        let count = inputs.uses.len();
        let growth = Growth::vector::<bool>(0, 0, count);
        facts.charge_caller_inputs(
            self.statements
                .len()
                .saturating_mul(
                    usize::try_from(count.max(1).ilog2())
                        .map_err(|_| "value reborrow index width overflow")?
                        + 1,
                )
                .saturating_add(growth.work)
                .saturating_add(count),
            growth.bytes,
        )?;
        let mut seen = Vec::new();
        seen.try_reserve(count)
            .map_err(|_| "value reborrow site scratch allocation failed")?;
        seen.resize(count, false);
        let mut previous = None;
        for row in &inputs.uses {
            if previous.is_some_and(|id| id >= row.operation.0) {
                return Err("value reborrow inputs are duplicate or unordered");
            }
            self.validate_value_reborrow_operand(*row)?;
            previous = Some(row.operation.0);
        }
        for statement in &self.statements {
            let super::ir::Action::Evaluate(id) = statement.action else {
                continue;
            };
            if !self
                .values
                .get(id.0 as usize)
                .is_some_and(|v| matches!(v.kind, Rvalue::Reborrow { .. }))
            {
                continue;
            }
            let index = inputs
                .uses
                .binary_search_by_key(&id.0, |row| row.operation.0)
                .map_err(|_| "value reborrow statement has no actual captured operation")?;
            if seen[index] || self.syntax_site(statement.source)? != inputs.uses[index].site {
                return Err("value reborrow changed its actual source statement");
            }
            seen[index] = true;
        }
        if seen.iter().any(|present| !*present) {
            return Err("value reborrow has no actual source statement");
        }
        Ok(())
    }
    pub(in crate::body) fn map_value_reborrow_inputs(
        &mut self,
        map: &mut impl FnMut(TypeId) -> Result<TypeId, &'static str>,
    ) -> Result<(), &'static str> {
        let Some(inputs) = self
            .resolved_callables
            .as_deref_mut()
            .and_then(|p| p.value_reborrows.as_deref_mut())
        else {
            return Ok(());
        };
        for row in &mut inputs.uses {
            row.source = map(row.source)?;
            row.target = map(row.target)?;
            if let Some(field) = &mut row.field {
                field.source_ty = map(field.source_ty)?;
                field.target_ty = map(field.target_ty)?;
            }
        }
        Ok(())
    }
}

pub(crate) fn region(types: &Store, ty: TypeId) -> Result<crate::types::RegionId, &'static str> {
    let Some(crate::types::Kind::Adt(_, args)) = types.kind(ty) else {
        return Err("value reborrow result is not a nominal application");
    };
    let mut regions = types
        .arguments(args)
        .ok_or("value reborrow result arguments disappeared")?
        .filter_map(|a| {
            if let crate::types::arguments::Argument::Lifetime(r) = a {
                Some(r)
            } else {
                None
            }
        });
    let first = regions
        .next()
        .ok_or("value reborrow result has no lifetime")?;
    if regions.next().is_some() {
        return Err("value reborrow result has multiple lifetimes");
    }
    Ok(first)
}
