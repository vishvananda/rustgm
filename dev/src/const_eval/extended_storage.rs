//! Initializer temporary storage reuses lexical extension, independently of promotion.
use super::{
    Failure, Frame, Machine,
    code::{Code, PreparedOperand},
};
use crate::body::{
    ir::{Body, Temporary, ValueId},
    scopes::Lifetime,
};

fn extended(body: &Body, temporary: &Temporary) -> bool {
    temporary.lifetime == Lifetime::Static
        && body
            .promotions
            .as_ref()
            .and_then(|p| p.get(temporary.value))
            .is_none()
}
fn allocation_bytes(count: usize) -> usize {
    if count == 0 {
        return 0;
    }
    count
        .saturating_mul(size_of::<ValueId>())
        .saturating_add(2 * size_of::<usize>())
        .next_multiple_of(align_of::<usize>())
}
impl Code {
    pub(super) fn extended_count(body: &Body) -> usize {
        body.temporaries
            .iter()
            .filter(|t| extended(body, t))
            .count()
    }
    pub(super) fn extended_storage_bound(body: &Body) -> usize {
        allocation_bytes(Self::extended_count(body))
    }
    pub(super) fn snapshot_extended_storage(
        body: &Body,
        operands: &mut Vec<PreparedOperand>,
    ) -> Result<(), &'static str> {
        let count = Self::extended_count(body);
        if count == 0 {
            return Ok(());
        }
        let mut values = Vec::new();
        values
            .try_reserve_exact(count)
            .map_err(|_| "extended temporary snapshot allocation failed")?;
        for temporary in body.temporaries.iter().filter(|t| extended(body, t)) {
            if body.values.get(temporary.value.0 as usize).is_none() {
                return Err("extended temporary has no actual typed value");
            }
            values.push(temporary.value);
        }
        values.sort_unstable_by_key(|id| id.0);
        values.dedup();
        operands.push(PreparedOperand::ExtendedStorage(values.into()));
        Ok(())
    }
    pub(super) fn extended_temporaries(&self) -> &[ValueId] {
        match self.operands.first() {
            Some(PreparedOperand::ExtendedStorage(values)) => values,
            _ => &[],
        }
    }
    pub(super) fn extended_storage_bytes(&self) -> usize {
        allocation_bytes(self.extended_temporaries().len())
    }
    pub(super) fn extends_temporary(&self, id: ValueId) -> bool {
        self.extended_temporaries()
            .binary_search_by_key(&id.0, |v| v.0)
            .is_ok()
    }
}
impl Machine<'_, '_> {
    /// These values were fully evaluated before the initializer borrowed them.
    /// Validate their backing after ordinary frame storage has expired too:
    /// an extended container cannot keep a reference to a dead local alive.
    pub(super) fn validate_extended_storage(
        &mut self,
        frame: &Frame,
        body: &Code,
    ) -> Result<(), Failure> {
        self.charge(body.extended_temporaries().len())?;
        for id in body.extended_temporaries() {
            let Some(address) = frame.temporaries.get(id).copied() else {
                continue;
            };
            let location = address
                .location()
                .ok_or("extended temporary has no storage identity")?;
            let place =
                crate::sema::values::Reader::new(self.evaluator.facts()).location(location)?;
            let value = place
                .value
                .ok_or("extended temporary lacks its evaluated contents")?;
            let ty = body
                .values
                .get(id.0 as usize)
                .ok_or("extended temporary lost its typed value")?
                .ty;
            if place.ty != ty {
                return Err("extended temporary changed its stored type".into());
            }
            self.validate_result(value, ty)?;
        }
        Ok(())
    }
}
