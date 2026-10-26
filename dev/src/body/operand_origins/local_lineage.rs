//! Demand-only physical lineage never establishes callable or source permission.
use super::{Origins, WORK_LIMIT};
use crate::{
    body::ir::{Action, Body, Place, Rvalue, ValueId},
    types::Store,
};

#[derive(Clone, Copy)]
enum Stored {
    Empty,
    One(ValueId),
    Unknown,
}

impl Origins<'_> {
    pub(crate) fn prepare_transport(&mut self, types: &Store) -> Result<(), &'static str> {
        self.prepare(types, |_| Ok(()))?;
        let count = self.body.values.len();
        if self.roots.len() == count.saturating_mul(2) {
            return Ok(());
        }
        let work = self
            .body
            .locals
            .len()
            .saturating_add(self.body.statements.len().saturating_mul(2))
            .saturating_add(self.body.projections.len())
            .saturating_add(count.saturating_mul(3));
        self.attempted = self.attempted.saturating_add(work);
        let scratch = self
            .body
            .locals
            .len()
            .saturating_mul(size_of::<Stored>())
            .saturating_add(size_of::<Vec<Stored>>());
        let payload = count.saturating_mul(2 * size_of::<ValueId>());
        let result = (|| {
            let previous = self.roots.capacity().saturating_mul(size_of::<ValueId>());
            if self.attempted > WORK_LIMIT
                || scratch.saturating_add(payload).saturating_add(previous) > WORK_LIMIT * 8
            {
                return Err("physical value origin work/storage budget exceeded");
            }
            let stores = unique_stores(self.body)?;
            let actual_scratch = stores
                .capacity()
                .saturating_mul(size_of::<Stored>())
                .saturating_add(size_of::<Vec<Stored>>());
            if actual_scratch
                .saturating_add(payload)
                .saturating_add(previous)
                > WORK_LIMIT * 8
            {
                return Err("physical value origin scratch coexistence budget exceeded");
            }
            self.roots
                .try_reserve_exact(count)
                .map_err(|_| "physical value origin allocation failed")?;
            self.storage = self.roots.capacity().saturating_mul(size_of::<ValueId>());
            if self
                .storage
                .saturating_add(actual_scratch)
                .saturating_add(previous)
                > WORK_LIMIT * 8
            {
                self.roots = Vec::new();
                return Err("physical value origin retained payload budget exceeded");
            }
            for (index, value) in self.body.values.iter().enumerate() {
                let own =
                    ValueId(u32::try_from(index).map_err(|_| "physical value identity exhausted")?);
                let edge = match value.kind {
                    Rvalue::Constrain(input) | Rvalue::Reify(input) => input,
                    Rvalue::Read(Place::Local(local)) => match stores.get(local.0 as usize) {
                        Some(Stored::One(input)) => *input,
                        _ => own,
                    },
                    _ => own,
                };
                if edge.0 as usize >= count {
                    return Err("physical value origin edge is outside its actual body");
                }
                self.roots.push(edge);
            }
            Ok(())
        })();
        if let Err(error) = result {
            self.failure = Some(error);
            self.complete = false;
            self.roots.clear();
            return Err(error);
        }
        Ok(())
    }

    pub(crate) fn transport_origin(
        &mut self,
        body: &Body,
        value: ValueId,
    ) -> Result<ValueId, &'static str> {
        if !std::ptr::eq(self.body, body)
            || !self.complete
            || self.failure.is_some()
            || self.roots.len() != body.values.len().saturating_mul(2)
        {
            return Err("physical value origins require their completed actual body");
        }
        self.attempted = self.attempted.saturating_add(1);
        if self.attempted > WORK_LIMIT {
            self.failure = Some("physical value origin query budget exceeded");
            return Err("physical value origin query budget exceeded");
        }
        self.roots
            .get(body.values.len().saturating_add(value.0 as usize))
            .copied()
            .ok_or("physical value origin is outside its prepared body")
    }
}

fn unique_stores(body: &Body) -> Result<Vec<Stored>, &'static str> {
    let mut stores = Vec::new();
    stores
        .try_reserve_exact(body.locals.len())
        .map_err(|_| "physical local origin scratch allocation failed")?;
    if stores.capacity().saturating_mul(size_of::<Stored>()) > WORK_LIMIT * 8 {
        return Err("physical local origin scratch capacity budget exceeded");
    }
    stores.resize(body.locals.len(), Stored::Empty);
    // A projected access or escaping address is deliberately outside this
    // closed unique-local profile. No indirect write may preserve these rows.
    if body.statements.iter().any(|s| {
        matches!(
            s.action,
            Action::Store {
                place: Place::Projection(_) | Place::Temporary(_) | Place::Static(_),
                ..
            }
        )
    }) || !body.projections.is_empty()
    {
        stores.fill(Stored::Unknown);
        return Ok(stores);
    }
    for statement in &body.statements {
        if let Action::Store {
            place: Place::Local(local),
            value,
        } = statement.action
        {
            let state = stores
                .get_mut(local.0 as usize)
                .ok_or("physical store local disappeared")?;
            *state = if matches!(state, Stored::Empty) {
                Stored::One(value)
            } else {
                Stored::Unknown
            };
        }
    }
    for value in &body.values {
        if let Rvalue::Address(Place::Local(local))
        | Rvalue::Borrow {
            place: Place::Local(local),
            ..
        } = value.kind
        {
            *stores
                .get_mut(local.0 as usize)
                .ok_or("physical borrowed local disappeared")? = Stored::Unknown;
        }
    }
    Ok(stores)
}
