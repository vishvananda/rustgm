pub trait Inspect {
    type Target;
    fn inspect(&self) -> &Self::Target;
}

pub struct Wrapper<T>(pub T);

impl<T> Inspect for Wrapper<T> {
    type Target = T;
    fn inspect(&self) -> &T {
        &self.0
    }
}

pub struct Indirect<P>(pub P);

impl<P: Inspect> Inspect for Indirect<P> {
    type Target = P::Target;
    fn inspect(&self) -> &P::Target {
        P::inspect(&self.0)
    }
}

pub fn indirect<P: Inspect>(value: &Indirect<P>) -> &P::Target {
    value.inspect()
}
