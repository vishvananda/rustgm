//! Only actual symbolic and concrete cleanup plans can enter the shared loan join.
use crate::borrow::destruction::{self, schedule::Schedule, symbolic};

mod sealed {
    pub trait Payload {}
    impl Payload for super::symbolic::Plan {}
    impl Payload for super::destruction::Plan {}
}

pub(crate) trait Payload: sealed::Payload {
    fn schedule(&self) -> Schedule<'_>;
}
impl Payload for symbolic::Plan {
    fn schedule(&self) -> Schedule<'_> {
        self.schedule()
    }
}
impl Payload for destruction::Plan {
    fn schedule(&self) -> Schedule<'_> {
        Schedule::concrete(self)
    }
}
