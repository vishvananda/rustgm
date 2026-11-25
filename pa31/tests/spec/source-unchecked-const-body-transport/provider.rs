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

impl Clone for u32 {
    fn clone(&self) -> Self {
        *self
    }
}
impl Copy for u32 {}

fn runtime_helper(x: u32) -> u32 {
    match x {
        10 => 17,
        5 => 15,
        _ => x,
    }
}

#[rustc_do_not_const_check]
pub const fn unchecked_free(x: u32) -> u32 {
    runtime_helper(x)
}

impl Clone for bool {
    fn clone(&self) -> Self {
        *self
    }
}
impl Copy for bool {}

impl<T: PointeeSized> Clone for *const T {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T: PointeeSized> Copy for *const T {}

#[lang = "not"]
pub const trait Not {
    type Output;
    fn not(self) -> Self::Output;
}
const impl Not for bool {
    type Output = bool;
    fn not(self) -> bool {
        !self
    }
}

#[lang = "eq"]
pub const trait PartialEq<Rhs: PointeeSized = Self>: PointeeSized {
    fn eq(&self, other: &Rhs) -> bool;
    fn ne(&self, other: &Rhs) -> bool {
        !self.eq(other)
    }
}
impl<T: PointeeSized> PartialEq for *const T {
    fn eq(&self, other: &*const T) -> bool {
        *self == *other
    }
}

fn runtime_pick<T: Copy>(x: T, y: T, pick: bool) -> T {
    if pick { x } else { y }
}

#[rustc_do_not_const_check]
pub const fn unchecked_generic<T: Copy>(x: T, y: T, pick: bool) -> T {
    runtime_pick(x, y, pick)
}

pub struct Wrap(pub u32);

impl Wrap {
    #[rustc_do_not_const_check]
    pub const fn unchecked_method(&self) -> u32 {
        runtime_helper(self.0)
    }
}

#[rustc_intrinsic]
#[rustc_nounwind]
#[rustc_do_not_const_check]
#[inline]
#[miri::intrinsic_fallback_is_spec]
pub const fn ptr_guaranteed_cmp<T>(ptr: *const T, other: *const T) -> u8 {
    (ptr == other) as u8
}

pub fn cmp_ptrs(a: *const u32, b: *const u32) -> u8 {
    ptr_guaranteed_cmp(a, b)
}

pub fn generic_ptr_eq<T>(a: *const T, b: *const T) -> bool {
    a == b
}

#[rustc_comptime]
pub fn comptime_seed() -> u32 {
    41
}
