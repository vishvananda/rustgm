#[lang = "freeze"]
pub unsafe auto trait Freeze {}

#[lang = "pin"]
#[repr(transparent)]
pub struct Pin<P> {
    pub pointer: P,
}

#[lang = "deref"]
pub trait Deref: PointeeSized {
    #[lang = "deref_target"]
    type Target: PointeeSized;

    fn deref(&self) -> &Self::Target;
}

impl<T: PointeeSized> Deref for &mut T {
    type Target = T;

    fn deref(&self) -> &T {
        &**self
    }
}

impl<T: PointeeSized> Deref for &T {
    type Target = T;

    fn deref(&self) -> &T {
        *self
    }
}

impl<P: Deref> Deref for Pin<P> {
    type Target = P::Target;

    fn deref(&self) -> &Self::Target {
        self.pointer.deref()
    }
}

impl<P: Deref> LegacyReceiver for Pin<P> {}

pub trait Resume {
    fn resume(self: Pin<&mut Self>, mask: u8) -> u8;
}

impl<'a> Resume for Pin<&'a mut u8> {
    fn resume(self: Pin<&mut Self>, mask: u8) -> u8 {
        match (*self.pointer.pointer, mask) {
            (42, 1) => 43,
            (7, 3) => 4,
            _ => 0,
        }
    }
}

pub struct Read<'a>(pub &'a u8);

impl<'a> Read<'a> {
    pub const fn read(self: Pin<&Self>) -> u8 {
        *self.pointer.0
    }
}
