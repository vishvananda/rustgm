#![feature(intrinsics,pattern_types,pattern_type_macro,decl_macro,marker_trait_attr,freeze_impls,allow_internal_unstable,staged_api)]
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
#[macro_export]
#[rustc_builtin_macro(pattern_type)]
#[unstable(feature = "pattern_type_macro", issue = "123646")]
macro_rules! pattern_type {
    ($($arg:tt)*) => {
        /* compiler built-in */
    };
}
pub mod mem { pub use crate::intrinsics::transmute; }
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
    use crate::{PointeeSized, Copy};
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
#[doc(hidden)]
#[allow(missing_debug_implementations)]
#[unstable(
    feature = "derive_clone_copy_internals",
    reason = "deriving hack, should not be public",
    issue = "none"
)]
pub struct AssertParamIsCopy<T: Copy + PointeeSized> {
    _field: crate::marker::PhantomData<T>,
}

    #[rustc_builtin_macro]
    #[stable(feature = "builtin_macro_prelude", since = "1.38.0")]
    #[allow_internal_unstable(core_intrinsics, derive_clone_copy_internals, trivial_clone)]
    pub macro Clone($item:item) {}
}

#[lang = "neg"]
#[stable(feature = "rust1", since = "1.0.0")]
#[rustc_const_unstable(feature = "const_ops", issue = "143802")]
#[doc(alias = "-")]
pub const trait Neg {
    /// The resulting type after applying the `-` operator.
    #[stable(feature = "rust1", since = "1.0.0")]
    type Output;

    /// Performs the unary `-` operation.
    ///
    /// # Example
    ///
    /// ```
    /// let x: i32 = 12;
    /// assert_eq!(-x, -12);
    /// ```
    #[must_use = "this returns the result of the operation, without modifying the original"]
    #[rustc_diagnostic_item = "neg"]
    #[stable(feature = "rust1", since = "1.0.0")]
    fn neg(self) -> Self::Output;
}

// Selected actual neg_impl expansion for i8; operator and body remain source owned.
#[stable(feature = "rust1", since = "1.0.0")]
#[rustc_const_unstable(feature = "const_ops", issue = "143802")]
const impl Neg for i8 {
    type Output = i8;
    #[inline]
    #[track_caller]
    #[rustc_inherit_overflow_checks]
    fn neg(self) -> i8 { -self }
}

// Authored primitive protocol instances use the actual source Clone body shape.
impl Clone for i8 { fn clone(&self)->Self { *self } }
impl Copy for i8 {}
impl Clone for char { fn clone(&self)->Self { *self } }
impl Copy for char {}

// Genuine pinned source Location declaration retains inherited track_caller.
pub mod location {
    use crate::{NonNull,PhantomData,derive};
    use crate::marker::Copy;use crate::clone::Clone;
#[lang = "panic_location"]
#[derive(Copy, Clone)]
#[stable(feature = "panic_hooks", since = "1.10.0")]
pub struct Location<'a> {
    // A raw pointer is used rather than a reference because the pointer is valid for one more byte
    // than the length stored in this pointer; the additional byte is the NUL-terminator used by
    // `Location::file_as_c_str`.
    filename: NonNull<str>,
    line: u32,
    col: u32,
    _filename: PhantomData<&'a str>,
}
}
#[unstable(feature = "pattern_type_range_trait", issue = "123646")]
#[rustc_const_unstable(feature = "pattern_type_range_trait", issue = "123646")]
#[diagnostic::on_unimplemented(
    message = "`{Self}` is not a valid base type for range patterns",
    label = "only integer types and `char` are supported"
)]
pub const trait RangePattern {
    /// Trait version of the inherent `MIN` assoc const.
    #[lang = "RangeMin"]
    const MIN: Self;

    /// Trait version of the inherent `MIN` assoc const.
    #[lang = "RangeMax"]
    const MAX: Self;

    /// A compile-time helper to subtract 1 for exclusive ranges.
    #[lang = "RangeSub"]
    #[track_caller]
    fn sub_one(self) -> Self;
}

// Authored, intentionally noncanonical source implementations. These are not
// the original checked_sub/char implementations, which remain required.
impl RangePattern for u8 {
    const MIN:Self=3;
    const MAX:Self=9;
    fn sub_one(self)->Self { self }
}
#[rustc_const_unstable(feature="pattern_type_range_trait",issue="123646")]
const impl RangePattern for i8 {
    const MIN:Self=-128;
    const MAX:Self=127;
    fn sub_one(self)->Self { self }
}
pub mod niche {
    use crate::{derive,pattern_type};use crate::clone::Clone;use crate::marker::Copy;
    #[derive(Clone,Copy)] #[repr(transparent)] pub struct OpenUpper(pattern_type!(u8 is 1..));
    #[derive(Clone,Copy)] #[repr(transparent)] pub struct OpenLower(pattern_type!(u8 is ..=7));
    #[derive(Clone,Copy)] #[repr(transparent)] pub struct Exclusive(pattern_type!(u8 is 3..7));
    #[derive(Clone,Copy)] #[repr(transparent)] pub struct Signed(pattern_type!(i8 is ..=-1 | 1..));
}
const U:niche::OpenUpper=unsafe{intrinsics::transmute(7u8)};
const L:niche::OpenLower=unsafe{intrinsics::transmute(3u8)};
const E:niche::Exclusive=unsafe{intrinsics::transmute(7u8)};
const S:niche::Signed=unsafe{intrinsics::transmute(-2i8)};
#[unsafe(no_mangle)]pub extern "C" fn observed_upper()->u8{unsafe{intrinsics::transmute(Clone::clone(&U))}}
#[unsafe(no_mangle)]pub extern "C" fn observed_lower()->u8{unsafe{intrinsics::transmute(Clone::clone(&L))}}
#[unsafe(no_mangle)]pub extern "C" fn observed_exclusive()->u8{unsafe{intrinsics::transmute(Clone::clone(&E))}}
#[unsafe(no_mangle)]pub extern "C" fn observed_signed()->i32{let x:i8=unsafe{intrinsics::transmute(Clone::clone(&S))};x as i32}
#[unsafe(no_mangle)]pub extern "C" fn observed_positive()->i32{let v:niche::Signed=unsafe{intrinsics::transmute(7i8)};let x:i8=unsafe{intrinsics::transmute(v)};x as i32}
