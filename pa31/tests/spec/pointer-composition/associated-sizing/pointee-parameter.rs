#![feature(trait_alias,extern_types,ptr_metadata,no_core,lang_items,rustc_attrs,intrinsics,core_intrinsics,auto_traits,staged_api,const_trait_impl,const_destruct,unsize,coerce_unsized)]
#![no_core]
#![unstable(feature="pointer_metadata_fixture",issue="none")]
#![allow(internal_features,dead_code)]
#[lang="pointee_sized"]pub trait PointeeSized{}
#[lang="meta_sized"]pub trait MetaSized:PointeeSized{}
#[lang="sized"]pub trait Sized:MetaSized{}
#[lang="legacy_receiver"]pub trait LegacyReceiver:PointeeSized{}
impl<T:PointeeSized>LegacyReceiver for &T{}
impl<T:PointeeSized>LegacyReceiver for &mut T{}
#[lang="clone"]pub trait Clone:Sized{#[lang="clone_fn"]fn clone(&self)->Self;}
#[lang="copy"]pub trait Copy:Clone{}
#[lang="freeze"]pub unsafe auto trait Freeze{}
#[unstable(feature="const_destruct",issue="133214")]
#[rustc_const_unstable(feature="const_destruct",issue="133214")]
#[lang="destruct"]
#[rustc_deny_explicit_impl]
#[rustc_dyn_incompatible_trait]
pub const trait Destruct:PointeeSized{}
#[unstable(feature="unsize",issue="18598")]
#[lang="unsize"]#[rustc_deny_explicit_impl]#[rustc_dyn_incompatible_trait]
pub trait Unsize<T:PointeeSized>:PointeeSized{}
#[unstable(feature="coerce_unsized",issue="18598")]
#[lang="coerce_unsized"]pub trait CoerceUnsized<T:PointeeSized>:Sized{}
#[unstable(feature="coerce_unsized",issue="18598")]
impl<T:PointeeSized+Unsize<U>,U:PointeeSized>CoerceUnsized<*const U> for *const T{}
impl<T:PointeeSized>Clone for *const T{fn clone(&self)->Self{*self}}
impl<T:PointeeSized>Copy for *const T{}
impl Clone for usize{fn clone(&self)->Self{*self}}
impl Copy for usize{}
impl Clone for u8{fn clone(&self)->Self{*self}}
impl Copy for u8{}
#[lang="pointee_trait"]pub trait Pointee:PointeeSized{
 #[lang="metadata_type"]type Metadata:Sized;
}
#[unstable(feature="core_intrinsics",issue="none")]
#[rustc_nounwind]
#[rustc_intrinsic_const_stable_indirect]
#[rustc_intrinsic]
pub const fn ptr_metadata<P:Pointee<Metadata=M>+PointeeSized,M>(ptr:*const P)->M;
// core::ptr::metadata's actual source wrapper body; generic result remains a projection.
pub const fn metadata<T:PointeeSized>(ptr:*const T)-><T as Pointee>::Metadata{ptr_metadata(ptr)}

impl<T:PointeeSized>Clone for *mut T{fn clone(&self)->Self{*self}}impl<T:PointeeSized>Copy for *mut T{}
pub mod intrinsics {
use crate::ptr;
pub mod bounds {
use crate::{PointeeSized,Sized};
pub unsafe trait BuiltinDeref:Sized {type Pointee:PointeeSized;}
unsafe impl<T:PointeeSized>BuiltinDeref for *const T{type Pointee=T;}
unsafe impl<T:PointeeSized>BuiltinDeref for *mut T{type Pointee=T;}
unsafe impl<T:PointeeSized>BuiltinDeref for &T{type Pointee=T;}
unsafe impl<T:PointeeSized>BuiltinDeref for &mut T{type Pointee=T;}
}
#[rustc_nounwind]
#[unstable(feature = "core_intrinsics", issue = "none")]
#[rustc_intrinsic_const_stable_indirect]
#[rustc_intrinsic]
pub const fn aggregate_raw_ptr<P: bounds::BuiltinDeref, D, M>(data: D, meta: M) -> P
where
    <P as bounds::BuiltinDeref>::Pointee: ptr::Pointee<Metadata = M>;
}
pub mod ptr {pub use crate::{Pointee,metadata};use crate::PointeeSized;use crate::intrinsics::aggregate_raw_ptr;
#[unstable(feature = "ptr_metadata", issue = "81513")]
// NOTE: don’t stabilize this before trait aliases are stable in the language?
pub trait Thin = Pointee<Metadata = ()> + PointeeSized;
/// Forms a (possibly-wide) raw pointer from a data pointer and metadata.
///
/// This function is safe but the returned pointer is not necessarily safe to dereference.
/// For slices, see the documentation of [`slice::from_raw_parts`] for safety requirements.
/// For trait objects, the metadata must come from a pointer to the same underlying erased type.
///
/// If you are attempting to deconstruct a DST in a generic context to be reconstructed later,
/// a thin pointer can always be obtained by casting `*const T` to `*const ()`.
///
/// [`slice::from_raw_parts`]: crate::slice::from_raw_parts
#[unstable(feature = "ptr_metadata", issue = "81513")]
#[inline]
pub const fn from_raw_parts<T: PointeeSized>(
    data_pointer: *const impl Thin,
    metadata: <T as Pointee>::Metadata,
) -> *const T {
    aggregate_raw_ptr(data_pointer, metadata)
}

/// Performs the same functionality as [`from_raw_parts`], except that a
/// raw `*mut` pointer is returned, as opposed to a raw `*const` pointer.
///
/// See the documentation of [`from_raw_parts`] for more details.
#[unstable(feature = "ptr_metadata", issue = "81513")]
#[inline]
pub const fn from_raw_parts_mut<T: PointeeSized>(
    data_pointer: *mut impl Thin,
    metadata: <T as Pointee>::Metadata,
) -> *mut T {
    aggregate_raw_ptr(data_pointer, metadata)
}

/// Forms a raw slice from a pointer and a length.
///
/// The `len` argument is the number of **elements**, not the number of bytes.
///
/// This function is safe, but actually using the return value is unsafe.
/// See the documentation of [`slice::from_raw_parts`] for slice safety requirements.
///
/// [`slice::from_raw_parts`]: crate::slice::from_raw_parts
///
/// # Examples
///
/// ```rust
/// use std::ptr;
///
/// // create a slice pointer when starting out with a pointer to the first element
/// let x = [5, 6, 7];
/// let raw_pointer = x.as_ptr();
/// let slice = ptr::slice_from_raw_parts(raw_pointer, 3);
/// assert_eq!(unsafe { &*slice }[2], 7);
/// ```
///
/// You must ensure that the pointer is valid and not null before dereferencing
/// the raw slice. A slice reference must never have a null pointer, even if it's empty.
///
/// ```rust,should_panic
/// use std::ptr;
/// let danger: *const [u8] = ptr::slice_from_raw_parts(ptr::null(), 0);
/// unsafe {
///     danger.as_ref().expect("references must not be null");
/// }
/// ```
#[inline]
#[stable(feature = "slice_from_raw_parts", since = "1.42.0")]
#[rustc_const_stable(feature = "const_slice_from_raw_parts", since = "1.64.0")]
#[rustc_diagnostic_item = "ptr_slice_from_raw_parts"]
pub const fn slice_from_raw_parts<T>(data: *const T, len: usize) -> *const [T] {
    from_raw_parts(data, len)
}

/// Forms a raw mutable slice from a pointer and a length.
///
/// The `len` argument is the number of **elements**, not the number of bytes.
///
/// Performs the same functionality as [`slice_from_raw_parts`], except that a
/// raw mutable slice is returned, as opposed to a raw immutable slice.
///
/// This function is safe, but actually using the return value is unsafe.
/// See the documentation of [`slice::from_raw_parts_mut`] for slice safety requirements.
///
/// [`slice::from_raw_parts_mut`]: crate::slice::from_raw_parts_mut
///
/// # Examples
///
/// ```rust
/// use std::ptr;
///
/// let x = &mut [5, 6, 7];
/// let raw_pointer = x.as_mut_ptr();
/// let slice = ptr::slice_from_raw_parts_mut(raw_pointer, 3);
///
/// unsafe {
///     (*slice)[2] = 99; // assign a value at an index in the slice
/// };
///
/// assert_eq!(unsafe { &*slice }[2], 99);
/// ```
///
/// You must ensure that the pointer is valid and not null before dereferencing
/// the raw slice. A slice reference must never have a null pointer, even if it's empty.
///
/// ```rust,should_panic
/// use std::ptr;
/// let danger: *mut [u8] = ptr::slice_from_raw_parts_mut(ptr::null_mut(), 0);
/// unsafe {
///     danger.as_mut().expect("references must not be null");
/// }
/// ```
#[inline]
#[stable(feature = "slice_from_raw_parts", since = "1.42.0")]
#[rustc_const_stable(feature = "const_slice_from_raw_parts_mut", since = "1.83.0")]
#[rustc_diagnostic_item = "ptr_slice_from_raw_parts_mut"]
pub const fn slice_from_raw_parts_mut<T>(data: *mut T, len: usize) -> *mut [T] {
    from_raw_parts_mut(data, len)
}

}

#[lang="sync"]pub unsafe auto trait Sync{}

trait Proj{type A:PointeeSized;}impl<T:PointeeSized>Proj for *const T{type A=T;}
