#[lang = "destruct"]
#[rustc_deny_explicit_impl]
#[rustc_dyn_incompatible_trait]
pub const trait Destruct: PointeeSized {}

#[lang = "freeze"]
pub unsafe auto trait Freeze {}
#[lang = "sync"]
pub unsafe auto trait Sync {}
#[cfg(rustc_oracle)]
#[lang = "drop_glue"]
pub unsafe fn drop_glue<T: PointeeSized>(_: &mut T) {}
#[cfg(rustc_oracle)]
#[lang = "future_trait"]
pub trait Future {
    type Output;
}

impl Clone for u32 {
    fn clone(&self) -> Self {
        *self
    }
}
impl Copy for u32 {}
impl<T: PointeeSized> Clone for &T {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T: PointeeSized> Copy for &T {}

pub trait Show {
    fn show(&self) -> u32;
}

struct Shown {
    value: u32,
}

impl Show for &Shown {
    fn show(&self) -> u32 {
        self.value
    }
}

pub struct Text {
    inner: Shown,
}

impl Text {
    pub const fn new(value: u32) -> Self {
        Self {
            inner: Shown { value },
        }
    }

    pub fn display(&self) -> impl Show {
        &self.inner
    }
}

pub struct Visible(pub u32);

impl Show for Visible {
    fn show(&self) -> u32 {
        self.0
    }
}

pub fn make_visible(value: u32) -> impl Show {
    Visible(value)
}

struct PairRef<'a, T>(&'a T);

impl<T: Show> Show for PairRef<'_, T> {
    fn show(&self) -> u32 {
        self.0.show()
    }
}

pub fn precise<'a, 'b, T: Show>(x: &'a T, _y: &'b u32) -> impl Show + use<'a, T> {
    PairRef(x)
}

pub const trait Value {
    fn get(&self) -> u32;
}

struct Boxed(u32);

const impl Value for Boxed {
    fn get(&self) -> u32 {
        self.0
    }
}

pub const fn const_box(value: u32) -> impl const Value + const Destruct {
    Boxed(value)
}

pub const fn const_maybe<V: [const] Value + [const] Destruct>(
    value: V,
) -> impl [const] Value + [const] Destruct {
    value
}

pub fn auto_bounded(value: u32) -> impl Show + Sync {
    Visible(value)
}
