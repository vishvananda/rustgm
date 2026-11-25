#[lang = "freeze"]
pub unsafe auto trait Freeze {}

pub trait Element: Copy {
    fn tag(self) -> u32;
}

impl Clone for u16 {
    fn clone(&self) -> Self {
        *self
    }
}
impl Copy for u16 {}
impl Clone for u32 {
    fn clone(&self) -> Self {
        *self
    }
}
impl Copy for u32 {}
impl<T: PointeeSized> Clone for *const T {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T: PointeeSized> Copy for *const T {}

impl Element for u8 {
    fn tag(self) -> u32 {
        self as u32
    }
}
impl Element for u16 {
    fn tag(self) -> u32 {
        self as u32
    }
}
impl Element for u32 {
    fn tag(self) -> u32 {
        self
    }
}

pub struct Lane<T: Element, const N: usize> {
    pub head: T,
}

pub struct PtrLane<T, const N: usize> {
    pub raw: T,
}

pub trait Castable {
    type Cast<T: Element>;
    type View<'a, T: Element + 'a>
    where
        Self: 'a;
    fn cast_tag<T: Element>(lane: Self::Cast<T>) -> u32;
}

pub trait CastablePtr {
    type CastPtr<U>;
}

impl<const N: usize> Castable for Lane<u32, N> {
    type Cast<T: Element> = Lane<T, N>;
    type View<'a, T: Element + 'a> = (&'a T, u32) where Self: 'a;

    fn cast_tag<T: Element>(lane: Self::Cast<T>) -> u32 {
        lane.head.tag()
    }
}

impl<T, const N: usize> CastablePtr for PtrLane<*const T, N> {
    type CastPtr<U> = PtrLane<*const U, N>;
}

pub fn read_view<'a, C: Castable, T: Element + 'a>(view: (&'a T, u32)) -> u32
where
    C: 'a,
{
    view.0.tag()
}
