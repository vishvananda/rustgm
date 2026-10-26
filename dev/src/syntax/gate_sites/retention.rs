//! Only real source transactions bind the cold identity and retention envelope.
use crate::support::source::OriginalImage;
use std::{cell::Cell, sync::Arc};
#[derive(Clone, Copy, Debug, Default)]
struct Work {
    steps: usize,
    bytes: usize,
}
#[derive(Clone, Debug, Default)]
pub(super) struct Witness {
    image: Option<Arc<OriginalImage>>,
    work: Cell<Work>,
}
impl Witness {
    pub(super) fn charge(&self, steps: usize, bytes: usize) -> Result<(), &'static str> {
        let mut work = self.work.get();
        work.steps = work.steps.saturating_add(steps);
        work.bytes = work.bytes.saturating_add(bytes);
        self.work.set(work);
        if self.exhausted() {
            return Err("defaultness retention work or storage budget exceeded");
        }
        Ok(())
    }
    pub(super) fn bound(&self) -> bool {
        self.image.is_some()
    }
    pub(super) fn exhausted(&self) -> bool {
        let work = self.work.get();
        work.steps > 10_000_000 || work.bytes > 64 * 1024 * 1024
    }
    pub(super) fn validate(&self, image: &Arc<OriginalImage>) -> Result<(), &'static str> {
        self.charge(1, 0)?;
        if self
            .image
            .as_ref()
            .is_none_or(|retained| !Arc::ptr_eq(retained, image))
        {
            return Err("defaultness source incarnation changed or was never bound");
        }
        Ok(())
    }
    pub(super) fn bind(&mut self, image: &Arc<OriginalImage>) -> Result<(), &'static str> {
        self.charge(1, 0)?;
        if self
            .image
            .as_ref()
            .is_some_and(|retained| !Arc::ptr_eq(retained, image))
        {
            return Err("defaultness source cannot be rebound by lookup");
        }
        self.commit(image);
        Ok(())
    }
    pub(super) fn commit(&mut self, image: &Arc<OriginalImage>) {
        if self.image.is_none() {
            self.image = Some(Arc::clone(image));
        }
    }
    pub(super) fn fresh(
        &mut self,
        previous: &Arc<OriginalImage>,
        image: &Arc<OriginalImage>,
    ) -> Result<(), &'static str> {
        self.validate(previous)?;
        self.image = Some(Arc::clone(image));
        Ok(())
    }
    #[cfg(test)]
    pub(super) fn usage(&self) -> (usize, usize) {
        let work = self.work.get();
        (work.steps, work.bytes)
    }
}
