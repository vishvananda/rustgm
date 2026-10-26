//! Indexed value origins describe checked coercions without paying their relations.
use super::ir::{Body, Rvalue, ValueId};
use crate::types::{Kind, Store, TypeId, instance::Instance};

const WORK_LIMIT: usize = 1_000_000;
mod local_lineage;

/// This cold index belongs to one immutable actual body. It is structural data,
/// not permission to execute that body or one of its callable values.
pub(crate) struct Origins<'body> {
    body: &'body Body,
    roots: Vec<ValueId>,
    attempted: usize,
    /// Observed vector payload capacity, excluding the Vec/Box and allocator
    /// metadata. An over-budget allocation is recorded before being released.
    storage: usize,
    complete: bool,
    failure: Option<&'static str>,
}

impl<'body> Origins<'body> {
    pub(crate) fn new(body: &'body Body) -> Self {
        Self {
            body,
            roots: Vec::new(),
            attempted: 0,
            storage: 0,
            complete: false,
            failure: None,
        }
    }

    /// Demand also charges its cumulative dependency owner. Emission has only
    /// this bounded per-body cold work/storage owner. Both retain failed work,
    /// and neither exposes a partially prepared table.
    pub(crate) fn prepare(
        &mut self,
        types: &Store,
        charge: impl FnOnce(usize) -> Result<(), &'static str>,
    ) -> Result<(), &'static str> {
        if let Some(error) = self.failure {
            return Err(error);
        }
        if self.complete {
            let result = charge(0);
            if let Err(error) = result {
                self.failure = Some(error);
                self.complete = false;
                self.roots.clear();
            }
            return result;
        }
        let count = self.body.values.len();
        self.attempted = self.attempted.saturating_add(count);
        let requested = count.saturating_mul(std::mem::size_of::<ValueId>());
        let result = (|| {
            charge(count)?;
            if self.attempted > WORK_LIMIT || requested > WORK_LIMIT * 4 {
                return Err("callable value origin work/storage budget exceeded");
            }
            let reserved = self.roots.try_reserve_exact(count);
            self.storage = self
                .roots
                .capacity()
                .saturating_mul(std::mem::size_of::<ValueId>());
            reserved.map_err(|_| "callable value origin storage allocation failed")?;
            if self.storage > WORK_LIMIT * 4 {
                self.roots = Vec::new();
                return Err("callable value origin retained payload budget exceeded");
            }
            for (index, value) in self.body.values.iter().enumerate() {
                let mut root =
                    ValueId(u32::try_from(index).map_err(|_| "too many callable value origins")?);
                if let Some(Kind::FunctionItem { item, .. }) = types.kind(value.ty)
                    && let Rvalue::Constrain(operand) = value.kind
                {
                    let source = self
                        .body
                        .values
                        .get(operand.0 as usize)
                        .filter(|_| (operand.0 as usize) < index)
                        .ok_or("callable constraint requires an earlier actual value")?;
                    if !matches!(types.kind(source.ty), Some(Kind::FunctionItem { item: actual, .. }) if actual == item)
                    {
                        return Err("callable constraint changed its actual declaration");
                    }
                    root = self.roots[operand.0 as usize];
                }
                self.roots.push(root);
            }
            Ok(())
        })();
        if let Err(error) = result {
            self.failure = Some(error);
            self.roots.clear();
            return Err(error);
        }
        self.complete = true;
        #[cfg(test)]
        OBSERVATIONS.with(|counts| {
            let (builds, rows, payload) = counts.get();
            counts.set((builds + 1, rows + count, payload.max(self.storage)));
        });
        Ok(())
    }

    pub(crate) fn origin(
        &self,
        types: &Store,
        body: &Body,
        value: ValueId,
    ) -> Result<ValueId, &'static str> {
        if !std::ptr::eq(self.body, body) {
            return Err("callable value origins belong to another actual body");
        }
        if let Some(error) = self.failure {
            return Err(error);
        }
        if !self.complete {
            return Err("callable value origins require completed preparation");
        }
        let origin = *self
            .roots
            .get(value.0 as usize)
            .ok_or("callable value origin is outside its actual body")?;
        let ty = self
            .body
            .values
            .get(origin.0 as usize)
            .ok_or("indexed callable value origin disappeared")?
            .ty;
        Instance::new(types, ty)?;
        Ok(origin)
    }
}

/// Both consumers retain the exact logical signature roots of the actual Call.
pub(crate) fn inputs(
    body: &Body,
    call: ValueId,
    types: [TypeId; 4],
) -> Result<[ValueId; 3], &'static str> {
    let value = body
        .values
        .get(call.0 as usize)
        .ok_or("source selector call disappeared")?;
    let Rvalue::Call { arguments, .. } = value.kind else {
        return Err("source selector requires an actual Call value");
    };
    let &[arg, constant, runtime] = body
        .arguments
        .get(arguments.start as usize..arguments.end as usize)
        .ok_or("source selector argument range disappeared")?
    else {
        return Err("source selector requires its three actual operands");
    };
    if value.ty != types[3] {
        return Err("source selector call result differs from its logical result");
    }
    for (operand, expected) in [arg, constant, runtime].into_iter().zip(types) {
        if body
            .values
            .get(operand.0 as usize)
            .is_none_or(|v| v.ty != expected)
        {
            return Err("source selector operand differs from its logical type argument");
        }
    }
    Ok([arg, constant, runtime])
}

#[cfg(test)]
std::thread_local! {
    static OBSERVATIONS: std::cell::Cell<(usize, usize, usize)> = const { std::cell::Cell::new((0, 0, 0)) };
}
#[cfg(test)]
pub(crate) fn take_observations() -> (usize, usize, usize) {
    OBSERVATIONS.with(std::cell::Cell::take)
}

#[cfg(test)]
#[path = "../../tests/invariants/operand_lineage.rs"]
mod tests;
