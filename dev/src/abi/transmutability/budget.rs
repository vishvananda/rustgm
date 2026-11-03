//! Every query pays before growth; rejected work remains visible to its source owner.
#[derive(Clone, Copy)]
pub struct Limits {
    pub work: usize,
    pub bytes: usize,
}
impl Default for Limits {
    fn default() -> Self {
        Self {
            work: 1_000_000,
            bytes: 16 * 1024 * 1024,
        }
    }
}
pub struct Budget<'a> {
    limits: Limits,
    pay: &'a mut dyn FnMut(usize, usize) -> Result<(), &'static str>,
    work: usize,
    bytes: usize,
}
impl<'a> Budget<'a> {
    pub fn new(
        limits: Limits,
        pay: &'a mut dyn FnMut(usize, usize) -> Result<(), &'static str>,
    ) -> Self {
        Self {
            limits,
            pay,
            work: 0,
            bytes: 0,
        }
    }
    #[must_use]
    pub const fn consumed(&self) -> (usize, usize) {
        (self.work, self.bytes)
    }
    pub(super) fn charge(&mut self, work: usize, bytes: usize) -> Result<(), &'static str> {
        self.work = self.work.saturating_add(work);
        self.bytes = self.bytes.saturating_add(bytes);
        (self.pay)(work, bytes)?;
        if self.work > self.limits.work || self.bytes > self.limits.bytes {
            return Err("transmutability query budget exceeded");
        }
        Ok(())
    }
}
pub(super) fn push<T>(
    values: &mut Vec<T>,
    value: T,
    budget: &mut Budget<'_>,
) -> Result<(), &'static str> {
    let growth = crate::support::retained::Growth::vector::<T>(values.len(), values.capacity(), 1);
    budget.charge(1 + growth.work, growth.bytes)?;
    values
        .try_reserve(1)
        .map_err(|_| "transmutability scratch allocation failed")?;
    values.push(value);
    Ok(())
}
