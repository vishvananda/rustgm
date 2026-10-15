//! Packed ancestor jumps keep captured-lifetime lookup logarithmic at arbitrary binder depth.
use super::{Constraints, Error, Scope};
impl Constraints<'_> {
    pub(super) fn prepare_ancestry(
        &mut self,
        parent: Option<Scope>,
    ) -> Result<(u16, usize), Error> {
        let depth = if let Some(parent) = parent {
            self.binders
                .get(parent.0)
                .ok_or(Error::Identity)?
                .depth
                .checked_add(1)
                .filter(|depth| *depth < 0x8000)
                .ok_or(Error::Budget)?
        } else {
            0
        };
        let start = self.ancestors.len();
        if depth > 1 {
            let mut halfway = parent.ok_or(Error::Identity)?;
            for power in 1..=depth.ilog2() {
                self.steps = self.steps.saturating_add(1);
                if self.steps >= self.limit {
                    return Err(Error::Budget);
                }
                halfway = self.jump(halfway, power - 1)?;
                self.ancestors.push(halfway);
            }
        }
        Ok((depth, start))
    }
    fn jump(&self, scope: Scope, power: u32) -> Result<Scope, Error> {
        let binder = self.binders.get(scope.0).ok_or(Error::Identity)?;
        if power >= 16 || u32::from(binder.depth) >> power == 0 {
            return Err(Error::Identity);
        }
        if power == 0 {
            return binder.parent.ok_or(Error::Identity);
        }
        self.ancestors
            .get(binder.jump_start + power as usize - 1)
            .copied()
            .ok_or(Error::Identity)
    }
    pub(super) fn ancestor(&self, mut scope: Scope, depth: u16) -> Result<Scope, Error> {
        let binder = self.binders.get(scope.0).ok_or(Error::Identity)?;
        if depth > binder.depth {
            return Err(Error::Identity);
        }
        if depth == binder.depth {
            return Ok(Scope(0));
        }
        let mut remaining = depth;
        while remaining != 0 {
            scope = self.jump(scope, remaining.trailing_zeros())?;
            remaining &= remaining - 1;
        }
        Ok(scope)
    }
}
