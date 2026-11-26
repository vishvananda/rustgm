#[lang = "freeze"]
pub unsafe auto trait Freeze {}

#[lang = "destruct"]
#[rustc_deny_explicit_impl]
#[rustc_dyn_incompatible_trait]
pub const trait Destruct: PointeeSized {}

#[cfg(rustc_oracle)]
#[lang = "drop_glue"]
pub unsafe fn drop_glue<T: PointeeSized>(_: &mut T) {}

impl Clone for u32 { fn clone(&self) -> Self { *self } }
impl Copy for u32 {}
impl<T: PointeeSized> Clone for &T { fn clone(&self) -> Self { *self } }
impl<T: PointeeSized> Copy for &T {}

pub const SELECT: fn(u32) -> u32 = |value| value;
pub const BORROW: for<'a> fn(&'a u32) -> &'a u32 = |value| value;
pub const VALUE: fn() -> u32 = || 19;
pub const BLOCK: fn() -> u32 = { || 29 };

pub fn with_check(value: u32) -> u32 {
    const _: fn(u32) = |_| {};
    value
}

pub fn borrowed(value: &u32) -> &u32 { (BORROW)(value) }
