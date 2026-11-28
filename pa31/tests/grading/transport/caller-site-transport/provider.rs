#![feature(intrinsics, pattern_types)]
#![feature(no_core,lang_items,rustc_attrs,fundamental,auto_traits,unboxed_closures,const_trait_impl)]
#![no_core]
#![allow(internal_features,dead_code,unused_variables)]
#[lang = "pointee_sized"]
#[diagnostic::on_unimplemented(
    message = "values of type `{Self}` may or may not have a size",
    label = "may or may not have a known size"
)]
#[fundamental]
#[rustc_specialization_trait]
#[rustc_deny_explicit_impl]
#[rustc_coinductive]
pub trait PointeeSized {
    // Empty
}
#[lang = "meta_sized"]
#[diagnostic::on_unimplemented(
    message = "the size for values of type `{Self}` cannot be known",
    label = "doesn't have a known size"
)]
#[fundamental]
#[rustc_specialization_trait]
#[rustc_deny_explicit_impl]
// `MetaSized` being coinductive, despite having supertraits, is okay for the same reasons as
// `Sized` above.
#[rustc_coinductive]
pub trait MetaSized: PointeeSized {
    // Empty
}
#[lang = "sized"]
#[diagnostic::on_unimplemented(
    message = "the size for values of type `{Self}` cannot be known at compilation time",
    label = "doesn't have a size known at compile-time"
)]
#[fundamental] // for Default, for example, which requires that `[T]: !Default` be evaluatable
#[rustc_specialization_trait]
#[rustc_deny_explicit_impl]
#[rustc_dyn_incompatible_trait]
// `Sized` being coinductive, despite having supertraits, is okay as there are no user-written impls,
// and we know that the supertraits are always implemented if the subtrait is just by looking at
// the builtin impls.
#[rustc_coinductive]
pub trait Sized: MetaSized {
    // Empty.
}
#[lang = "legacy_receiver"]
pub trait LegacyReceiver: PointeeSized {}
impl<T: PointeeSized> LegacyReceiver for &T {}
impl<T: PointeeSized> LegacyReceiver for &mut T {}
#[lang = "clone"]
pub trait Clone: Sized {  #[lang = "clone_fn"] fn clone(&self) -> Self; }
#[lang = "copy"]
pub trait Copy: Clone {}


#[lang = "freeze"]
pub unsafe auto trait Freeze {}
// Oracle-only shim: rustc's drop elaboration of the owned FnMut/Fn argument
// needs this item. RustGM never configures `rustc_oracle`.
#[cfg(rustc_oracle)]
#[lang = "drop_glue"]
pub unsafe fn drop_glue<T: PointeeSized>(_: &mut T) {}
impl Clone for bool { fn clone(&self) -> Self { *self } }
impl Copy for bool {}
impl Clone for u32 { fn clone(&self) -> Self { *self } }
impl Copy for u32 {}

#[rustc_builtin_macro(pattern_type)]
#[macro_export]
macro_rules! pattern_type { ($($args:tt)*) => {}; }

#[repr(transparent)]
#[rustc_nonnull_optimization_guaranteed]
#[rustc_diagnostic_item = "NonNull"]
pub struct NonNull<T: PointeeSized> {
    pointer: crate::pattern_type!(*const T is !null),
}

#[lang = "phantom_data"]
pub struct PhantomData<T: PointeeSized>;

#[lang = "panic_location"]
pub struct Location<'a> {
    filename: NonNull<str>,
    line: u32,
    col: u32,
    _filename: PhantomData<&'a str>,
}

pub mod intrinsics {
    #[rustc_intrinsic_const_stable_indirect]
    #[rustc_nounwind]
    #[rustc_intrinsic]
    pub const fn caller_location() -> &'static crate::Location<'static>;
}

impl<'a> Location<'a> {
    #[must_use]
    #[track_caller]
    #[inline]
    pub const fn caller() -> &'static Location<'static> {
        crate::intrinsics::caller_location()
    }

    pub const fn line(&self) -> u32 { self.line }
    pub const fn column(&self) -> u32 { self.col }
}

#[track_caller]
const fn private_tracked() -> &'static Location<'static> {
    Location::caller()
}

#[track_caller]
pub const fn inner_tracked<T>() -> &'static Location<'static> {
    private_tracked()
}

#[track_caller]
pub const fn outer_tracked<T>() -> &'static Location<'static> {
    inner_tracked::<T>()
}

pub const fn untracked_wrapper<T>() -> &'static Location<'static> {
    inner_tracked::<T>()
}

pub const fn branch_caller(first: bool) -> &'static Location<'static> {
    if first {
        Location::caller()
    } else {
        inner_tracked::<u32>()
    }
}

pub trait Locate {
    fn locate() -> &'static Location<'static>;
}

pub struct TrackedImpl;
pub struct UntrackedImpl;

impl Locate for TrackedImpl {
    #[track_caller]
    fn locate() -> &'static Location<'static> {
        Location::caller()
    }
}

impl Locate for UntrackedImpl {
    fn locate() -> &'static Location<'static> {
        Location::caller()
    }
}

pub fn call_locate<L: Locate>() -> &'static Location<'static> {
    L::locate()
}
