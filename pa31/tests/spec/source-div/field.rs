#![feature(core_intrinsics,intrinsics,pattern_types,decl_macro,marker_trait_attr,freeze_impls,allow_internal_unstable,staged_api)]
#![unstable(feature="location_fixture",issue="none")]
#![rustc_coherence_is_core]#![feature(no_core,lang_items,rustc_attrs,const_trait_impl,const_destruct,fundamental,auto_traits)]
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
#[doc(hidden)]
pub trait LegacyReceiver: PointeeSized {
    // Empty.
}

impl<T: PointeeSized> LegacyReceiver for &T {}

impl<T: PointeeSized> LegacyReceiver for &mut T {}
#[unstable(feature = "const_destruct", issue = "133214")]
#[rustc_const_unstable(feature = "const_destruct", issue = "133214")]
#[lang = "destruct"]
#[diagnostic::on_unimplemented(message = "can't drop `{Self}`")]
#[rustc_deny_explicit_impl]
#[rustc_dyn_incompatible_trait]
pub const trait Destruct: PointeeSized {}
#[lang = "freeze"]
pub unsafe auto trait Freeze {}
#[stable(feature = "rust1", since = "1.0.0")]
#[lang = "clone"]
#[rustc_diagnostic_item = "Clone"]
#[rustc_trivial_field_reads]
#[rustc_const_unstable(feature = "const_clone", issue = "142757")]
pub const trait Clone: Sized {
    /// Returns a duplicate of the value.
    ///
    /// Note that what "duplicate" means varies by type:
    /// - For most types, this creates a deep, independent copy
    /// - For reference types like `&T`, this creates another reference to the same value
    /// - For smart pointers like [`Arc`] or [`Rc`], this increments the reference count
    ///   but still points to the same underlying data
    ///
    /// [`Arc`]: ../../std/sync/struct.Arc.html
    /// [`Rc`]: ../../std/rc/struct.Rc.html
    ///
    /// # Examples
    ///
    /// ```
    /// # #![allow(noop_method_call)]
    /// let hello = "Hello"; // &str implements Clone
    ///
    /// assert_eq!("Hello", hello.clone());
    /// ```
    ///
    /// Example with a reference-counted type:
    ///
    /// ```
    /// use std::sync::{Arc, Mutex};
    ///
    /// let data = Arc::new(Mutex::new(vec![1, 2, 3]));
    /// let data_clone = data.clone(); // Creates another Arc pointing to the same Mutex
    ///
    /// {
    ///     let mut lock = data.lock().unwrap();
    ///     lock.push(4);
    /// }
    ///
    /// // Changes are visible through the clone because they share the same underlying data
    /// assert_eq!(*data_clone.lock().unwrap(), vec![1, 2, 3, 4]);
    /// ```
    #[must_use = "cloning is often expensive and is not expected to have side effects"]
    // Clone::clone is special because the compiler generates MIR to implement it for some types.
    // See InstanceKind::CloneShim.
    #[stable(feature = "rust1", since = "1.0.0")]
    #[lang = "clone_fn"]
    fn clone(&self) -> Self;

    /// Performs copy-assignment from `source`.
    ///
    /// `a.clone_from(&b)` is equivalent to `a = b.clone()` in functionality,
    /// but can be overridden to reuse the resources of `a` to avoid unnecessary
    /// allocations.
    #[inline]
    #[stable(feature = "rust1", since = "1.0.0")]
    fn clone_from(&mut self, source: &Self)
    where
        Self: [const] Destruct,
    {
        *self = source.clone()
    }
}
#[stable(feature = "rust1", since = "1.0.0")]
#[lang = "copy"]
#[rustc_diagnostic_item = "Copy"]
pub trait Copy: Clone {
    // Empty.
}
impl Clone for u8 { fn clone(&self)->Self { *self } }
impl Copy for u8 {}
#[inline(always)]
#[rustc_diagnostic_item = "convert_identity"]
pub const fn identity<T>(x: T) -> T {
    x
}

#[lang="drop_glue"]pub(crate) const unsafe fn drop_glue<T:PointeeSized>(_:&mut T)where T:[const]Destruct{}


pub mod intrinsics {
#![unstable(
    feature = "core_intrinsics",
    reason = "intrinsics are unlikely to ever be stabilized, instead \
                      they should be used through stabilized interfaces \
                      in the rest of the standard library",
    issue = "none"
)]
pub use crate::layout_intrinsics::{size_of,align_of};
    #[rustc_intrinsic_const_stable_indirect]
    #[rustc_nounwind]
    #[rustc_intrinsic]
    pub const fn caller_location() -> &'static crate::panic::Location<'static>;
    #[stable(feature = "rust1", since = "1.0.0")]
    #[rustc_const_stable(feature = "const_transmute", since = "1.56.0")]
    #[rustc_intrinsic]pub const unsafe fn transmute<Src,Dst>(src:Src)->Dst;
}


// Actual source raw-pointer copying declarations (core marker.rs/clone.rs).
#[stable(feature = "rust1", since = "1.0.0")]
#[rustc_const_unstable(feature = "const_clone", issue = "142757")]
const impl<T:PointeeSized> Clone for *const T { fn clone(&self)->Self { *self } }
impl<T:PointeeSized> Copy for *const T {}
#[stable(feature = "rust1", since = "1.0.0")]
#[rustc_const_unstable(feature = "const_clone", issue = "142757")]
const impl<T:PointeeSized> Clone for *mut T { fn clone(&self)->Self { *self } }
impl<T:PointeeSized> Copy for *mut T {}
// Authored reduced real Pointee/Metadata owner, independently qualified by C;
// complete original Debug/Copy/Send/Sync/Ord/Hash/Unpin/Freeze bounds remain required.
#[lang="pointee_trait"]
#[rustc_deny_explicit_impl]
#[rustc_dyn_incompatible_trait]
pub trait Pointee: PointeeSized {
    #[lang="metadata_type"] type Metadata: Sized;
}

// Pinned core::pat compiler declaration; no ordinary macro body is substituted.
#[rustc_builtin_macro(pattern_type)]
#[macro_export]
macro_rules! pattern_type { ($($args:tt)*) => {}; }
pub mod mem { pub use crate::intrinsics::transmute; pub use crate::mem_layout::size_of; }
use crate::intrinsics::transmute;
// Rust88d9 ptr/non_null.rs70–76: stability inventory is separate from this local provider.
#[repr(transparent)]
#[rustc_nonnull_optimization_guaranteed]
#[rustc_diagnostic_item = "NonNull"]
pub struct NonNull<T: PointeeSized> {
    pointer: crate::pattern_type!(*const T is !null),
}
impl<T: PointeeSized> Clone for NonNull<T> { fn clone(&self)->Self { *self } }
impl<T: PointeeSized> Copy for NonNull<T> {}
// Actual const method bodies retain their source paths and unsafe operations.
impl<T: PointeeSized> NonNull<T> {
    pub const fn from_ref(r: &T) -> Self {
        // SAFETY: A reference cannot be null.
        unsafe { transmute(r as *const T) }
    }
    #[rustc_never_returns_null_ptr]
    #[must_use]
    #[inline(always)]
    pub const fn as_ptr(self) -> *mut T {
        // This is a transmute for the same reasons as `NonZero::get`.
        // SAFETY: `NonNull` is transparent over a *const T.
        unsafe { mem::transmute::<Self, *mut T>(self) }
    }
    #[must_use]
    #[inline(always)]
    pub const unsafe fn as_ref<'a>(&self) -> &'a T {
        // SAFETY: the caller must guarantee the reference requirements.
        unsafe { &*self.as_ptr().cast_const() }
    }
    #[must_use]
    #[inline]
    pub const fn cast<U>(self) -> NonNull<U> {
        unsafe { transmute(self.as_ptr() as *mut U) }
    }
}
// Actual core::ptr::mut_ptr cast_const body, rather than a compiler library algorithm.
impl<T: PointeeSized> *mut T {
    pub const fn cast_const(self) -> *const T { self as _ }
}

// Real source unit marker; its argument remains logical and storage stays empty.
#[lang = "phantom_data"]
pub struct PhantomData<T: PointeeSized>;
impl<T: PointeeSized> Copy for PhantomData<T> {}
impl<T: PointeeSized> Clone for PhantomData<T> { fn clone(&self)->Self { *self } }
unsafe impl<T: PointeeSized> Freeze for PhantomData<T> {}
impl Clone for u32 { fn clone(&self)->Self { *self } }
impl Copy for u32 {}

// Genuine source compiler declarations; placeholder macro bodies are never run.
#[stable(feature = "rust1", since = "1.0.0")]
#[rustc_builtin_macro]
pub macro derive($item:item) {}
pub mod marker {
    pub use crate::{Copy, PhantomData};
    #[rustc_builtin_macro]
    #[stable(feature = "builtin_macro_prelude", since = "1.38.0")]
    #[allow_internal_unstable(core_intrinsics, derive_clone_copy_internals)]
    pub macro Copy($item:item) {}
}
pub mod clone {
    use crate::PointeeSized;
    pub use crate::Clone;
    #[unstable(feature = "trivial_clone", reason = "this isn't part of any API guarantee", issue = "none")]
    #[rustc_const_unstable(feature = "const_clone", issue = "142757")]
    #[lang = "trivial_clone"]
    #[rustc_unsafe_specialization_marker]
    #[marker]
    pub const unsafe trait TrivialClone: [const] Clone {}
    #[doc(hidden)]
    #[allow(missing_debug_implementations)]
    #[unstable(feature = "derive_clone_copy_internals", reason = "deriving hack, should not be public", issue = "none")]
    pub struct AssertParamIsClone<T: Clone + PointeeSized> {
        _field: crate::marker::PhantomData<T>,
    }
    #[rustc_builtin_macro]
    #[stable(feature = "builtin_macro_prelude", since = "1.38.0")]
    #[allow_internal_unstable(core_intrinsics, derive_clone_copy_internals, trivial_clone)]
    pub macro Clone($item:item) {}
}
pub mod location {
    use crate::{derive, NonNull, PhantomData};
    use crate::marker::Copy;
    use crate::clone::Clone;
    // Selected pinned panic/location.rs38-46 declaration and derive list.
    #[lang = "panic_location"]
    #[derive(Copy, Clone)]
    #[stable(feature = "panic_hooks", since = "1.10.0")]
    pub struct Location<'a> {
        filename: NonNull<str>,
        line: u32,
        col: u32,
        _filename: PhantomData<&'a str>,
    }
    // Authored test helper constructs the genuine private representation with
    // a three-byte filename and its additional readable NUL byte.
    pub fn observed() -> (u32,u32) {
        let bytes = b"loc\0";
        let raw = unsafe { crate::intrinsics::transmute::<(*const u8,usize),*const str>(
            (bytes as *const [u8;4] as *const u8, 3usize)) };
        let filename = unsafe { crate::intrinsics::transmute::<*const str,NonNull<str>>(raw) };
        let actual = Location { filename, line: 31u32, col: 7u32, _filename: PhantomData };
        let copied = actual;
        let cloned = actual.clone();
        (copied.line, cloned.col)
    }
    impl<'a> Location<'a> {
        #[must_use]
        #[stable(feature = "panic_hooks", since = "1.10.0")]
        #[rustc_const_stable(feature = "const_location_fields", since = "1.79.0")]
        #[inline]
        pub const fn line(&self) -> u32 { self.line }
        #[must_use]
        #[stable(feature = "panic_col", since = "1.25.0")]
        #[rustc_const_stable(feature = "const_location_fields", since = "1.79.0")]
        #[inline]
        pub const fn column(&self) -> u32 { self.col }
        #[must_use]
        #[stable(feature = "panic_hooks", since = "1.10.0")]
        #[rustc_const_stable(feature = "const_location_fields", since = "1.79.0")]
        pub const fn file(&self) -> &'a str {
            // SAFETY: The filename is valid.
            unsafe { self.filename.as_ref() }
        }
    }
    // Authored observer bridge reads the genuine private filename pointer.
    pub fn filename_parts(at: &Location<'static>) -> (*const u8,usize) {
        unsafe { crate::intrinsics::transmute::<*mut str,(*const u8,usize)>(at.filename.as_ptr()) }
    }

}
#[unsafe(no_mangle)] pub extern "C" fn location_line() -> u32 { location::observed().0 }
#[unsafe(no_mangle)] pub extern "C" fn location_column() -> u32 { location::observed().1 }

// Original Location methods, selected from pinned panic/location.rs.
pub mod panic { pub use crate::location::Location; }
impl<'a> location::Location<'a> {
    #[must_use]
    #[stable(feature = "track_caller", since = "1.46.0")]
    #[rustc_const_stable(feature = "const_caller_location", since = "1.79.0")]
    #[track_caller]
    #[inline]
    pub const fn caller() -> &'static location::Location<'static> {
        crate::intrinsics::caller_location()
    }
}
// Private fields keep their actual defining-module ownership.

// Actual pinned marker.rs Freeze indirection family, required by static locations.
#[unstable(feature = "freeze", issue = "121675")]
unsafe impl<T:PointeeSized> Freeze for *const T {}
#[unstable(feature = "freeze", issue = "121675")]
unsafe impl<T:PointeeSized> Freeze for *mut T {}
#[unstable(feature = "freeze", issue = "121675")]
unsafe impl<T:PointeeSized> Freeze for &T {}
#[unstable(feature = "freeze", issue = "121675")]
unsafe impl<T:PointeeSized> Freeze for &mut T {}

impl Clone for usize{fn clone(&self)->Self{*self}}impl Copy for usize{}
impl Clone for u16{fn clone(&self)->Self{*self}}impl Copy for u16{}
impl Clone for u64{fn clone(&self)->Self{*self}}impl Copy for u64{}
impl Clone for u128{fn clone(&self)->Self{*self}}impl Copy for u128{}
impl Clone for isize{fn clone(&self)->Self{*self}}impl Copy for isize{}
impl Clone for i8{fn clone(&self)->Self{*self}}impl Copy for i8{}
impl Clone for i16{fn clone(&self)->Self{*self}}impl Copy for i16{}
impl Clone for i32{fn clone(&self)->Self{*self}}impl Copy for i32{}
impl Clone for i64{fn clone(&self)->Self{*self}}impl Copy for i64{}
impl Clone for i128{fn clone(&self)->Self{*self}}impl Copy for i128{}
impl Clone for f32{fn clone(&self)->Self{*self}}impl Copy for f32{}
impl Clone for f64{fn clone(&self)->Self{*self}}impl Copy for f64{}
#[lang = "div"]
#[stable(feature = "rust1", since = "1.0.0")]
#[rustc_const_unstable(feature = "const_ops", issue = "143802")]
#[diagnostic::on_unimplemented(
    message = "cannot divide `{Self}` by `{Rhs}`",
    label = "no implementation for `{Self} / {Rhs}`"
)]
#[doc(alias = "/")]
pub const trait Div<Rhs = Self> {
    /// The resulting type after applying the `/` operator.
    #[stable(feature = "rust1", since = "1.0.0")]
    type Output;

    /// Performs the `/` operation.
    ///
    /// # Example
    ///
    /// ```
    /// assert_eq!(12 / 2, 6);
    /// ```
    #[must_use = "this returns the result of the operation, without modifying the original"]
    #[rustc_diagnostic_item = "div"]
    #[stable(feature = "rust1", since = "1.0.0")]
    fn div(self, rhs: Rhs) -> Self::Output;
}


macro_rules! forward_ref_binop {
    (impl $imp:ident, $method:ident for $t:ty, $u:ty, $(#[$attr:meta])+) => {
        $(#[$attr])+
        const impl $imp<$u> for &$t {
            type Output = <$t as $imp<$u>>::Output;

            #[inline]
            #[track_caller]
            fn $method(self, other: $u) -> <$t as $imp<$u>>::Output {
                $imp::$method(*self, other)
            }
        }

        $(#[$attr])+
        const impl $imp<&$u> for $t {
            type Output = <$t as $imp<$u>>::Output;

            #[inline]
            #[track_caller]
            fn $method(self, other: &$u) -> <$t as $imp<$u>>::Output {
                $imp::$method(self, *other)
            }
        }

        $(#[$attr])+
        const impl $imp<&$u> for &$t {
            type Output = <$t as $imp<$u>>::Output;

            #[inline]
            #[track_caller]
            fn $method(self, other: &$u) -> <$t as $imp<$u>>::Output {
                $imp::$method(*self, *other)
            }
        }
    }
}
macro_rules! div_impl_integer {
    ($(($($t:ty)*) => $panic:expr),*) => ($($(
        /// This operation rounds towards zero, truncating any
        /// fractional part of the exact result.
        ///
        /// # Panics
        ///
        #[doc = $panic]
        #[stable(feature = "rust1", since = "1.0.0")]
        #[rustc_const_unstable(feature = "const_ops", issue = "143802")]
        const impl Div for $t {
            type Output = $t;

            #[inline]
            #[track_caller]
            fn div(self, other: $t) -> $t { self / other }
        }

        forward_ref_binop! { impl Div, div for $t, $t,
        #[stable(feature = "rust1", since = "1.0.0")]
        #[rustc_const_unstable(feature = "const_ops", issue = "143802")] }
    )*)*)
}

div_impl_integer! {
    (usize u8 u16 u32 u64 u128) => "This operation will panic if `other == 0`.",
    (isize i8 i16 i32 i64 i128) => "This operation will panic if `other == 0` or the division results in overflow."
}


macro_rules! div_impl_float {
    ($($t:ty)*) => ($(
        #[stable(feature = "rust1", since = "1.0.0")]
        #[rustc_const_unstable(feature = "const_ops", issue = "143802")]
        const impl Div for $t {
            type Output = $t;

            #[inline]
            fn div(self, other: $t) -> $t { self / other }
        }

        forward_ref_binop! { impl Div, div for $t, $t,
        #[stable(feature = "rust1", since = "1.0.0")]
        #[rustc_const_unstable(feature = "const_ops", issue = "143802")] }
    )*)
}

// Explicit selected float family; original macro transcriber retained.
div_impl_float!{f32 f64}

// Actual source layout declarations reexported through the original intrinsic path.
pub mod layout_intrinsics{#[rustc_nounwind]
#[unstable(feature = "core_intrinsics", issue = "none")]
#[rustc_intrinsic_const_stable_indirect]
#[rustc_intrinsic]
pub const fn size_of<T>() -> usize;
#[rustc_nounwind]
#[unstable(feature = "core_intrinsics", issue = "none")]
#[rustc_intrinsic_const_stable_indirect]
#[rustc_intrinsic]
pub const fn align_of<T>() -> usize;
}
#[doc(hidden)]
#[unstable(feature = "sized_type_properties", issue = "none")]
pub trait SizedTypeProperties: Sized {
    #[doc(hidden)]
    #[unstable(feature = "sized_type_properties", issue = "none")]
    #[lang = "mem_size_const"]
    const SIZE: usize = intrinsics::size_of::<Self>();

    #[doc(hidden)]
    #[unstable(feature = "sized_type_properties", issue = "none")]
    #[lang = "mem_align_const"]
    const ALIGN: usize = intrinsics::align_of::<Self>();


}
#[doc(hidden)]
#[unstable(feature = "sized_type_properties", issue = "none")]
impl<T> SizedTypeProperties for T {}

pub mod mem_layout{use super::{SizedTypeProperties,intrinsics};
#[inline(always)]
#[must_use]
#[stable(feature = "rust1", since = "1.0.0")]
#[rustc_promotable]
#[rustc_const_stable(feature = "const_mem_size_of", since = "1.24.0")]
#[rustc_diagnostic_item = "mem_size_of"]
pub const fn size_of<T>() -> usize {
    // By making this a constant, we also guarantee that the constant can be successfully evaluated
    // in any program execution that actually executes `size_of`. Which is relevant because the
    // constant can fail to evaluate if the type is too big. Someone might do something cursed where
    // soundness relies on a certain type not being too big, and they check that by just invoking
    // size_of on the type to ensure it exists, so if we fully DCE'd size_of calls that would be
    // considered unsound... but by making this a constant, it participates in the usual "required
    // consts" system, and we are safe.
    <T as SizedTypeProperties>::SIZE
}
}
pub struct PtrWords {pub data:[*const ();16 / mem::size_of::<*const ()>()]}
#[unsafe(no_mangle)]pub extern "C" fn pointer_words_size()->usize{mem::size_of::<PtrWords>()}
