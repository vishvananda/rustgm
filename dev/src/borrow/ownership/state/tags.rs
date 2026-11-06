//! Cleanup tag bits use only the may-plane, never physical read/write permission.
use super::{Maybe, State};
impl<I: Maybe> State<I> {
    pub(in crate::borrow::ownership) fn possible(
        &mut self,
        bit: u32,
    ) -> Result<bool, &'static str> {
        self.charge(1)?;
        let word = self
            .words
            .get(bit as usize / 64)
            .ok_or("cleanup tag bit is outside its prepaid table")?;
        Ok(word.initialized.bits() & (1_u64 << (bit % 64)) != 0)
    }
    pub(in crate::borrow::ownership) fn set_possible(
        &mut self,
        start: u32,
        end: u32,
        value: bool,
    ) -> Result<(), &'static str> {
        if start > end {
            return Err("cleanup tag interval is reversed");
        }
        let mut bit = start;
        while bit < end {
            self.charge(1)?;
            let width = (end - bit).min(64 - bit % 64);
            let mask = (u64::MAX >> (64 - width)) << (bit % 64);
            let word = self
                .words
                .get_mut(bit as usize / 64)
                .ok_or("cleanup tag write is outside its prepaid table")?;
            word.initialized.assign(mask, value);
            bit += width;
        }
        Ok(())
    }
}
