//! Executable storage consumes the same completed static solution as callable targets.
use super::{Code, PreparedOperand, Rvalue};
use crate::types::{Kind, Store, TypeId, instance::Instance, substitute::Bound};

pub(super) fn closure_entry(
    types: &mut Store,
    fold: &mut Bound,
    entry: crate::body::invocation_entries::Entry,
    work: &mut usize,
) -> Result<crate::body::invocation_entries::Entry, &'static str> {
    let mut restore = Restore { types, fold, work };
    let instance = restore.storage(entry.instance().ty())?;
    let receiver = entry
        .adapter()
        .map(|adapter| restore.storage(adapter.receiver()))
        .transpose()?;
    let execution = restore.signature(entry.execution())?;
    entry.stored(
        restore.types,
        Instance::closure(restore.types, instance)?,
        execution,
        receiver,
    )
}

struct Restore<'a> {
    types: &'a mut Store,
    fold: &'a mut Bound,
    work: &'a mut usize,
}
impl Code {
    pub(super) fn restore_storage(
        &mut self,
        types: &mut Store,
        fold: &mut Bound,
        work: &mut usize,
    ) -> Result<(), &'static str> {
        if self.cleanup().is_some() {
            return Err("constant cleanup snapshots cannot be restored into another Code");
        }
        self.require_reborrow_context(types)?;
        let mut restore = Restore { types, fold, work };
        self.result = restore.storage(self.result)?;
        for local in &mut self.locals {
            local.ty = restore.storage(local.ty)?;
        }
        for projection in &mut self.projections {
            projection.ty = restore.storage(projection.ty)?;
        }
        for value in &mut self.values {
            value.ty = restore.storage(value.ty)?;
            match &mut value.kind {
                Rvalue::Call { signature, .. } | Rvalue::Invoke { signature, .. } => {
                    *signature = restore.signature(*signature)?;
                }
                Rvalue::Intrinsic { instance, .. } => {
                    // This is operation data, not a source callable's captured context.
                    let ty = restore.apply(instance.ty())?;
                    *instance = Instance::new(restore.types, ty)?;
                }
                Rvalue::Unsize { conversion, .. } => {
                    *conversion = conversion.try_map(|ty| restore.storage(ty))?;
                }
                _ => {}
            }
        }
        for operand in &mut self.operands {
            if let PreparedOperand::Reborrows(inputs) = operand {
                std::sync::Arc::get_mut(inputs)
                    .ok_or("shared value reborrow snapshot cannot be restored")?
                    .map_types(&mut |ty| restore.storage(ty))?;
            }
            if let PreparedOperand::Inline { result, .. } = operand {
                *result = restore.storage(*result)?;
            }
        }
        Ok(())
    }
}
impl Restore<'_> {
    fn apply(&mut self, ty: TypeId) -> Result<TypeId, &'static str> {
        *self.work = self.work.saturating_add(1);
        let before = self.fold.steps;
        let result = self.fold.apply(self.types, ty);
        *self.work = self
            .work
            .saturating_add(self.fold.steps.saturating_sub(before));
        result
    }
    fn storage(&mut self, ty: TypeId) -> Result<TypeId, &'static str> {
        if self
            .types
            .contains_function_items(ty)
            .ok_or("proved storage lost its canonical type")?
        {
            // Source item contexts remain raw; Code's separate map owns their VM targets.
            *self.work = self.work.saturating_add(1);
            return Ok(ty);
        }
        self.apply(ty)
    }
    fn signature(&mut self, ty: TypeId) -> Result<TypeId, &'static str> {
        *self.work = self.work.saturating_add(1);
        let Some(Kind::Function {
            bound_regions: 0,
            parameters,
            result,
            unsafe_,
            convention,
        }) = self.types.kind(ty)
        else {
            return Err("proved executable call retains an unopened signature");
        };
        // Call roots already live in this body: its empty signature binder adds no scope.
        let count = self
            .types
            .list(parameters)
            .ok_or("proved call lost its parameter list")?
            .len();
        let mut changed: Option<Vec<TypeId>> = None;
        for index in 0..count {
            let original = self
                .types
                .list(parameters)
                .ok_or("proved call lost its parameters")?[index];
            let actual = self.storage(original)?;
            if actual != original && changed.is_none() {
                let mut children = Vec::new();
                children
                    .try_reserve_exact(count)
                    .map_err(|_| "proved call parameter allocation failed")?;
                children.extend_from_slice(
                    self.types
                        .list(parameters)
                        .ok_or("proved call lost its parameters")?,
                );
                changed = Some(children);
            }
            if let Some(children) = &mut changed {
                children[index] = actual;
            }
        }
        let actual_result = self.storage(result)?;
        if changed.is_none() && result == actual_result {
            return Ok(ty);
        }
        let parameters = if let Some(children) = changed {
            self.types.intern_list(&children)?
        } else {
            parameters
        };
        self.types.intern(Kind::Function {
            bound_regions: 0,
            parameters,
            result: actual_result,
            unsafe_,
            convention,
        })
    }
}
