#[lang = "freeze"]
pub unsafe auto trait Freeze {}

#[lang = "deref"]
pub trait Deref: PointeeSized {
    #[lang = "deref_target"]
    type Target: PointeeSized;

    fn deref(&self) -> &Self::Target;
}

impl<T: PointeeSized> Deref for &T {
    type Target = T;

    fn deref(&self) -> &T {
        *self
    }
}

impl<T: PointeeSized> Deref for &mut T {
    type Target = T;

    fn deref(&self) -> &T {
        &**self
    }
}

pub struct Byte<'a, const N: usize>(pub &'a u8);

pub trait Advance {
    fn advance<'a>(self: &mut &'a Self, next: &'a Self) -> u8;
    fn advance_mut<'a>(self: &mut &'a mut Self, next: &'a mut Self) -> u8;
}

impl<const N: usize> Advance for Byte<'_, N> {
    fn advance<'a>(self: &mut &'a Self, next: &'a Self) -> u8 {
        let previous = *self.0;
        *self = next;
        previous
    }

    fn advance_mut<'a>(self: &mut &'a mut Self, next: &'a mut Self) -> u8 {
        let previous = *self.0;
        *self = next;
        previous
    }
}
