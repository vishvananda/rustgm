//! One cold state belongs to this exact frame and once-bound executable footer.
use super::{Bundle, Rc};
use crate::{
    body::drops::schedule::{Effect, Guard},
    const_eval::{Constant, Failure, Frame, Machine, code::Code},
};
use std::num::NonZeroU32;
#[cfg(test)]
#[path = "../../../tests/invariants/ctmask.rs"]
pub(in crate::const_eval) mod tests;
pub(in crate::const_eval) struct Cursor {
    pub(super) event: Option<NonZeroU32>,
    pub(super) parent: Option<NonZeroU32>,
    instance: usize,
    owner: Option<Rc<Bundle>>,
    flags: Vec<u64>,
}
impl Cursor {
    pub(in crate::const_eval) fn slots(
        code: &Code,
        parent: Option<NonZeroU32>,
    ) -> Result<usize, Failure> {
        if code.cleanup().is_none() && parent.is_none() {
            return Ok(0);
        }
        let words = code
            .cleanup()
            .map_or(0, |b| (b.view.flag_count as usize).div_ceil(64));
        let bytes = words
            .checked_mul(size_of::<u64>())
            .and_then(|n| n.checked_add(size_of::<Self>()))
            .ok_or("constant cleanup frame storage overflow")?;
        Ok(bytes.div_ceil(size_of::<Constant>()))
    }
    pub(in crate::const_eval) fn new(
        code: &Code,
        instance: usize,
        parent: Option<NonZeroU32>,
    ) -> Result<Option<Box<Self>>, Failure> {
        if code.cleanup().is_none() && parent.is_none() {
            return Ok(None);
        }
        let owner = code.cleanup().cloned();
        if let Some(owner) = &owner {
            owner.require_code(code)?;
        }
        let words = owner
            .as_ref()
            .map_or(0, |b| (b.view.flag_count as usize).div_ceil(64));
        let mut flags = Vec::new();
        flags
            .try_reserve_exact(words)
            .map_err(|_| "constant cleanup frame allocation failed")?;
        flags.resize(words, 0);
        if size_of::<Self>()
            .saturating_add(flags.capacity().saturating_mul(size_of::<u64>()))
            .div_ceil(size_of::<Constant>())
            != Self::slots(code, parent)?
        {
            return Err("cleanup frame allocation exceeded its prepaid storage".into());
        }
        Ok(Some(Box::new(Self {
            event: None,
            parent,
            instance,
            owner,
            flags,
        })))
    }
    pub(in crate::const_eval) fn retained_slots(&self) -> usize {
        size_of::<Self>()
            .saturating_add(self.flags.capacity().saturating_mul(size_of::<u64>()))
            .div_ceil(size_of::<Constant>())
    }
    fn require(&self, code: &Code, instance: usize) -> Result<&Bundle, Failure> {
        if self.instance != instance {
            return Err("cleanup state belongs to another actual frame".into());
        }
        let owner = self
            .owner
            .as_ref()
            .ok_or("cleanup cursor has no actual footer owner")?;
        if code
            .cleanup()
            .is_none_or(|current| !Rc::ptr_eq(current, owner))
        {
            return Err("cleanup state belongs to another actual footer".into());
        }
        owner.require_code(code)?;
        if self.flags.len() != (owner.view.flag_count as usize).div_ceil(64) {
            return Err("cleanup state changed its exact flag storage extent".into());
        }
        let remainder = owner.view.flag_count % 64;
        if remainder != 0 && self.flags.last().is_some_and(|word| word >> remainder != 0) {
            return Err("cleanup state has bits outside its actual flag domain".into());
        }
        Ok(owner)
    }
    pub(super) fn require_frame(&self, code: &Code, instance: usize) -> Result<(), Failure> {
        self.require(code, instance).map(|_| ())
    }
    pub(super) fn allowed(
        &self,
        code: &Code,
        instance: usize,
        event: usize,
    ) -> Result<bool, Failure> {
        let owner = self.require(code, instance)?;
        if self.event.map(|i| (i.get() - 1) as usize) != Some(event) {
            return Err("cleanup guard differs from its actual current event".into());
        }
        let Effect::Cleanup(index) = owner
            .view
            .events
            .get(event)
            .ok_or("invalid cleanup guard event")?
            .effect
        else {
            return Err("cleanup guard does not belong to an actual call event".into());
        };
        let call = owner
            .view
            .call(index)?
            .ok_or("cleanup guard lost its actual active call")?;
        self.bit(call.guard, owner.view.flag_count)
    }
    fn bit(&self, guard: Guard, count: u32) -> Result<bool, Failure> {
        let Some(index) = guard.index() else {
            return Ok(true);
        };
        if index >= count as usize {
            return Err("cleanup guard exceeds its actual flag domain".into());
        }
        let word = self
            .flags
            .get(index / 64)
            .ok_or("missing cleanup flag word")?;
        Ok(word & (1u64 << (index % 64)) != 0)
    }
    pub(super) fn write(
        &mut self,
        code: &Code,
        instance: usize,
        event: usize,
    ) -> Result<usize, Failure> {
        let owner = self.require(code, instance)?;
        if self.event.map(|i| (i.get() - 1) as usize) != Some(event) {
            return Err("cleanup write differs from its actual current event".into());
        }
        let Effect::Flags { range, initialized } = owner
            .view
            .events
            .get(event)
            .ok_or("invalid flag write event")?
            .effect
        else {
            return Err("cleanup flag write lost its actual event".into());
        };
        if range.start >= range.end || range.end > owner.view.flag_count {
            return Err("cleanup flag write exceeds its actual plan domain".into());
        }
        for index in range.start..range.end {
            let word = self
                .flags
                .get_mut(index as usize / 64)
                .ok_or("missing cleanup flag word")?;
            let mask = 1u64 << (index % 64);
            if initialized {
                *word |= mask;
            } else {
                *word &= !mask;
            }
        }
        Ok((range.end - range.start) as usize)
    }
}
impl Frame {
    pub(in crate::const_eval) fn cleanup_slots(&self) -> usize {
        self.cleanup
            .as_ref()
            .map_or(0, |state| state.retained_slots())
    }
    pub(in crate::const_eval) fn cleanup_allowed(
        &self,
        code: &Code,
        event: usize,
    ) -> Result<bool, Failure> {
        self.cleanup
            .as_ref()
            .ok_or("cleanup has no actual frame state")?
            .allowed(code, self.instance, event)
    }
    pub(in crate::const_eval) fn reset_cleanup_event(&mut self) {
        if let Some(state) = &mut self.cleanup {
            state.event = None;
        }
    }
}

impl Machine<'_, '_> {
    pub(in crate::const_eval) fn prepare_cleanup(
        &mut self,
        code: &Code,
        parent: Option<NonZeroU32>,
        slots: usize,
    ) -> Result<Option<Box<Cursor>>, Failure> {
        // Storage is zeroed once; real entry events initialize ABI parameter bits.
        let words = code
            .cleanup()
            .map_or(0, |b| (b.view.flag_count as usize).div_ceil(64));
        self.charge(words.saturating_add(usize::from(slots != 0)))?;
        Cursor::new(code, self.work.calls, parent)
    }
}
