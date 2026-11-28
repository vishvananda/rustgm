#![feature(auto_traits, intrinsics)]
// Authored Option, Result, and Ordering declarations with the pinned role shapes.
#![feature(try_blocks)]
#![feature(unboxed_closures)]
#![feature(no_core,lang_items,rustc_attrs,const_trait_impl)]
#![no_core]
#![allow(internal_features,dead_code,unused_variables)]
#[lang="pointee_sized"]pub trait PointeeSized{}
#[lang="meta_sized"]pub trait MetaSized:PointeeSized{}
#[lang="sized"]pub trait Sized:MetaSized{}
#[lang="legacy_receiver"]pub trait LegacyReceiver:PointeeSized{}
impl<T:PointeeSized> LegacyReceiver for &T{}
impl<T:PointeeSized> LegacyReceiver for &mut T{}
#[lang="clone"]pub trait Clone:Sized{#[lang="clone_fn"]fn clone(&self)->Self;}
#[lang="copy"]pub trait Copy:Clone{}
impl Clone for u8{fn clone(&self)->Self{*self}}impl Copy for u8{}
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

#[rustc_intrinsic]
pub fn abort() -> !;

#[rustc_intrinsic]
#[miri::intrinsic_fallback_is_spec]
#[cold]
pub const fn cold_path() {}

#[rustc_intrinsic]
pub const unsafe fn transmute_unchecked<Src, Dst>(src: Src) -> Dst;

#[rustc_intrinsic]
pub const fn ctpop<T: Copy>(x: T) -> u32;

#[rustc_intrinsic]
pub const fn sub_with_overflow<T: Copy>(x: T, y: T) -> (T, bool);

#[rustc_intrinsic]
pub const unsafe fn unchecked_sub<T: Copy>(x: T, y: T) -> T;

#[rustc_intrinsic]
pub const fn black_box<T>(dummy: T) -> T;

#[inline(always)]
#[rustc_intrinsic]
pub const fn ub_checks() -> bool {
    false
}

#[rustc_intrinsic]
pub const fn forget<T>(_: T);

#[rustc_intrinsic]
#[miri::intrinsic_fallback_is_spec]
pub const fn select_unpredictable<T>(b: bool, true_val: T, false_val: T) -> T {
    if b {
        forget(false_val);
        true_val
    } else {
        forget(true_val);
        false_val
    }
}

#[rustc_comptime]
pub fn comptime_free() -> u32 {
    41
}

pub struct Holder(pub u32);

impl Holder {
    #[rustc_comptime]
    pub fn comptime_inner(self) -> u32 {
        self.0
    }
}

pub fn pop_count(x: u32) -> u32 {
    ctpop(x)
}

pub fn transmute_u32(x: u32) -> u32 {
    unsafe { transmute_unchecked(x) }
}

pub fn black_boxed(x: u32) -> u32 {
    black_box(x)
}

pub fn unchecked_diff(a: u32, b: u32) -> u32 {
    unsafe { unchecked_sub(a, b) }
}

pub fn overflow_diff(a: u32, b: u32) -> (u32, bool) {
    sub_with_overflow(a, b)
}

pub fn select_hint(flag: bool, yes: u32, no: u32) -> u32 {
    select_unpredictable(flag, yes, no)
}

pub fn hinted<T: Copy>(value: T) -> T {
    cold_path();
    value
}

pub fn maybe_abort(flag: bool) -> u32 {
    if flag {
        abort()
    }
    0
}
