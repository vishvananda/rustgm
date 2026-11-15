#![feature(core_intrinsics)]
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

// Authored signed zero-niche controls. Each endpoint implementation is canonical for its
// integer; an inclusive end never requests `sub_one`. The Option roles, layout intrinsics,
// primitive copies and negations follow the existing source foundations.
#[lang="Option"]pub enum Option<T>{#[lang="None"]None,#[lang="Some"]Some(T)}
use Option::{None,Some};
#[rustc_nounwind]#[unstable(feature="core_intrinsics",issue="none")]#[rustc_intrinsic_const_stable_indirect]#[rustc_intrinsic]pub const fn size_of<T>()->usize;
#[rustc_nounwind]#[unstable(feature="core_intrinsics",issue="none")]#[rustc_intrinsic_const_stable_indirect]#[rustc_intrinsic]pub const fn align_of<T>()->usize;
#[rustc_const_unstable(feature="pattern_type_range_trait",issue="123646")]
const impl RangePattern for i8{const MIN:Self=-128;const MAX:Self=127;fn sub_one(self)->Self{self}}
impl Clone for i16{fn clone(&self)->Self{*self}}impl Copy for i16{}
#[stable(feature="rust1",since="1.0.0")]#[rustc_const_unstable(feature="const_ops",issue="143802")]
const impl Neg for i16{type Output=i16;#[inline]#[track_caller]#[rustc_inherit_overflow_checks]fn neg(self)->i16{-self}}
#[rustc_const_unstable(feature="pattern_type_range_trait",issue="123646")]
const impl RangePattern for i16{const MIN:Self=-32768;const MAX:Self=32767;fn sub_one(self)->Self{self}}
impl Clone for i32{fn clone(&self)->Self{*self}}impl Copy for i32{}
#[stable(feature="rust1",since="1.0.0")]#[rustc_const_unstable(feature="const_ops",issue="143802")]
const impl Neg for i32{type Output=i32;#[inline]#[track_caller]#[rustc_inherit_overflow_checks]fn neg(self)->i32{-self}}
#[rustc_const_unstable(feature="pattern_type_range_trait",issue="123646")]
const impl RangePattern for i32{const MIN:Self=-2147483648;const MAX:Self=2147483647;fn sub_one(self)->Self{self}}
impl Clone for i64{fn clone(&self)->Self{*self}}impl Copy for i64{}
#[stable(feature="rust1",since="1.0.0")]#[rustc_const_unstable(feature="const_ops",issue="143802")]
const impl Neg for i64{type Output=i64;#[inline]#[track_caller]#[rustc_inherit_overflow_checks]fn neg(self)->i64{-self}}
#[rustc_const_unstable(feature="pattern_type_range_trait",issue="123646")]
const impl RangePattern for i64{const MIN:Self=-9223372036854775808;const MAX:Self=9223372036854775807;fn sub_one(self)->Self{self}}
impl Clone for i128{fn clone(&self)->Self{*self}}impl Copy for i128{}
#[stable(feature="rust1",since="1.0.0")]#[rustc_const_unstable(feature="const_ops",issue="143802")]
const impl Neg for i128{type Output=i128;#[inline]#[track_caller]#[rustc_inherit_overflow_checks]fn neg(self)->i128{-self}}
#[rustc_const_unstable(feature="pattern_type_range_trait",issue="123646")]
const impl RangePattern for i128{const MIN:Self=-170141183460469231731687303715884105728;const MAX:Self=170141183460469231731687303715884105727;fn sub_one(self)->Self{self}}
impl Clone for isize{fn clone(&self)->Self{*self}}impl Copy for isize{}
#[stable(feature="rust1",since="1.0.0")]#[rustc_const_unstable(feature="const_ops",issue="143802")]
const impl Neg for isize{type Output=isize;#[inline]#[track_caller]#[rustc_inherit_overflow_checks]fn neg(self)->isize{-self}}
#[rustc_const_unstable(feature="pattern_type_range_trait",issue="123646")]
const impl RangePattern for isize{const MIN:Self=-9223372036854775808;const MAX:Self=9223372036854775807;fn sub_one(self)->Self{self}}

#[derive(crate::clone::Clone,crate::marker::Copy)]#[repr(transparent)]pub struct Nz_i8(pattern_type!(i8 is ..=-1|1..));
impl Nz_i8{
 pub const fn new(val:i8)->Option<Self>{#[allow(non_contiguous_range_endpoints)]if let ..=-1|1.. = val{Some(unsafe{Nz_i8(intrinsics::transmute(val))})}else{None}}
 pub const unsafe fn new_unchecked(val:i8)->Self{unsafe{intrinsics::transmute(val)}}
 pub const fn as_inner(self)->i8{unsafe{intrinsics::transmute(self)}}
}
const NONE_BITS_i8:i8=unsafe{intrinsics::transmute::<Option<Nz_i8>,i8>(None)};
const SOME_BITS_i8:i8=unsafe{intrinsics::transmute::<Option<Nz_i8>,i8>(Some(Nz_i8::new_unchecked(1i8)))};
const MINUS_BITS_i8:i8=unsafe{intrinsics::transmute::<Option<Nz_i8>,i8>(Some(Nz_i8::new_unchecked(-1i8)))};
const FROM_BITS_i8:Option<Nz_i8>=unsafe{intrinsics::transmute::<i8,Option<Nz_i8>>(-128i8)};
#[unsafe(no_mangle)]pub extern "C" fn echo_i8(x:Option<Nz_i8>)->Option<Nz_i8>{x}
#[unsafe(no_mangle)]pub extern "C" fn make_i8(x:i8)->Option<Nz_i8>{Nz_i8::new(x)}
#[unsafe(no_mangle)]pub extern "C" fn get_i8(x:Option<Nz_i8>)->i8{match x{Some(x)=>x.as_inner(),None=>0}}
#[unsafe(no_mangle)]pub extern "C" fn invoke_i8(x:Option<Nz_i8>)->Option<Nz_i8>{let f:extern "C" fn(Option<Nz_i8>)->Option<Nz_i8>=echo_i8;f(x)}
#[unsafe(no_mangle)]pub extern "C" fn size_i8()->usize{const{size_of::<Option<Nz_i8>>()}}
#[unsafe(no_mangle)]pub extern "C" fn align_i8()->usize{const{align_of::<Option<Nz_i8>>()}}
#[unsafe(no_mangle)]pub extern "C" fn constant_none_i8()->i8{NONE_BITS_i8}
#[unsafe(no_mangle)]pub extern "C" fn constant_some_i8()->i8{SOME_BITS_i8}
#[unsafe(no_mangle)]pub extern "C" fn constant_minus_i8()->i8{MINUS_BITS_i8}
#[unsafe(no_mangle)]pub extern "C" fn constant_round_i8()->i8{get_i8(FROM_BITS_i8)}

#[derive(crate::clone::Clone,crate::marker::Copy)]#[repr(transparent)]pub struct Nz_i16(pattern_type!(i16 is ..=-1|1..));
impl Nz_i16{
 pub const fn new(val:i16)->Option<Self>{#[allow(non_contiguous_range_endpoints)]if let ..=-1|1.. = val{Some(unsafe{Nz_i16(intrinsics::transmute(val))})}else{None}}
 pub const unsafe fn new_unchecked(val:i16)->Self{unsafe{intrinsics::transmute(val)}}
 pub const fn as_inner(self)->i16{unsafe{intrinsics::transmute(self)}}
}
const NONE_BITS_i16:i16=unsafe{intrinsics::transmute::<Option<Nz_i16>,i16>(None)};
const SOME_BITS_i16:i16=unsafe{intrinsics::transmute::<Option<Nz_i16>,i16>(Some(Nz_i16::new_unchecked(1i16)))};
const MINUS_BITS_i16:i16=unsafe{intrinsics::transmute::<Option<Nz_i16>,i16>(Some(Nz_i16::new_unchecked(-1i16)))};
const FROM_BITS_i16:Option<Nz_i16>=unsafe{intrinsics::transmute::<i16,Option<Nz_i16>>(-32768i16)};
#[unsafe(no_mangle)]pub extern "C" fn echo_i16(x:Option<Nz_i16>)->Option<Nz_i16>{x}
#[unsafe(no_mangle)]pub extern "C" fn make_i16(x:i16)->Option<Nz_i16>{Nz_i16::new(x)}
#[unsafe(no_mangle)]pub extern "C" fn get_i16(x:Option<Nz_i16>)->i16{match x{Some(x)=>x.as_inner(),None=>0}}
#[unsafe(no_mangle)]pub extern "C" fn invoke_i16(x:Option<Nz_i16>)->Option<Nz_i16>{let f:extern "C" fn(Option<Nz_i16>)->Option<Nz_i16>=echo_i16;f(x)}
#[unsafe(no_mangle)]pub extern "C" fn size_i16()->usize{const{size_of::<Option<Nz_i16>>()}}
#[unsafe(no_mangle)]pub extern "C" fn align_i16()->usize{const{align_of::<Option<Nz_i16>>()}}
#[unsafe(no_mangle)]pub extern "C" fn constant_none_i16()->i16{NONE_BITS_i16}
#[unsafe(no_mangle)]pub extern "C" fn constant_some_i16()->i16{SOME_BITS_i16}
#[unsafe(no_mangle)]pub extern "C" fn constant_minus_i16()->i16{MINUS_BITS_i16}
#[unsafe(no_mangle)]pub extern "C" fn constant_round_i16()->i16{get_i16(FROM_BITS_i16)}

#[derive(crate::clone::Clone,crate::marker::Copy)]#[repr(transparent)]pub struct Nz_i32(pattern_type!(i32 is ..=-1|1..));
impl Nz_i32{
 pub const fn new(val:i32)->Option<Self>{#[allow(non_contiguous_range_endpoints)]if let ..=-1|1.. = val{Some(unsafe{Nz_i32(intrinsics::transmute(val))})}else{None}}
 pub const unsafe fn new_unchecked(val:i32)->Self{unsafe{intrinsics::transmute(val)}}
 pub const fn as_inner(self)->i32{unsafe{intrinsics::transmute(self)}}
}
const NONE_BITS_i32:i32=unsafe{intrinsics::transmute::<Option<Nz_i32>,i32>(None)};
const SOME_BITS_i32:i32=unsafe{intrinsics::transmute::<Option<Nz_i32>,i32>(Some(Nz_i32::new_unchecked(1i32)))};
const MINUS_BITS_i32:i32=unsafe{intrinsics::transmute::<Option<Nz_i32>,i32>(Some(Nz_i32::new_unchecked(-1i32)))};
const FROM_BITS_i32:Option<Nz_i32>=unsafe{intrinsics::transmute::<i32,Option<Nz_i32>>(-2147483648i32)};
#[unsafe(no_mangle)]pub extern "C" fn echo_i32(x:Option<Nz_i32>)->Option<Nz_i32>{x}
#[unsafe(no_mangle)]pub extern "C" fn make_i32(x:i32)->Option<Nz_i32>{Nz_i32::new(x)}
#[unsafe(no_mangle)]pub extern "C" fn get_i32(x:Option<Nz_i32>)->i32{match x{Some(x)=>x.as_inner(),None=>0}}
#[unsafe(no_mangle)]pub extern "C" fn invoke_i32(x:Option<Nz_i32>)->Option<Nz_i32>{let f:extern "C" fn(Option<Nz_i32>)->Option<Nz_i32>=echo_i32;f(x)}
#[unsafe(no_mangle)]pub extern "C" fn size_i32()->usize{const{size_of::<Option<Nz_i32>>()}}
#[unsafe(no_mangle)]pub extern "C" fn align_i32()->usize{const{align_of::<Option<Nz_i32>>()}}
#[unsafe(no_mangle)]pub extern "C" fn constant_none_i32()->i32{NONE_BITS_i32}
#[unsafe(no_mangle)]pub extern "C" fn constant_some_i32()->i32{SOME_BITS_i32}
#[unsafe(no_mangle)]pub extern "C" fn constant_minus_i32()->i32{MINUS_BITS_i32}
#[unsafe(no_mangle)]pub extern "C" fn constant_round_i32()->i32{get_i32(FROM_BITS_i32)}

#[derive(crate::clone::Clone,crate::marker::Copy)]#[repr(transparent)]pub struct Nz_i64(pattern_type!(i64 is ..=-1|1..));
impl Nz_i64{
 pub const fn new(val:i64)->Option<Self>{#[allow(non_contiguous_range_endpoints)]if let ..=-1|1.. = val{Some(unsafe{Nz_i64(intrinsics::transmute(val))})}else{None}}
 pub const unsafe fn new_unchecked(val:i64)->Self{unsafe{intrinsics::transmute(val)}}
 pub const fn as_inner(self)->i64{unsafe{intrinsics::transmute(self)}}
}
const NONE_BITS_i64:i64=unsafe{intrinsics::transmute::<Option<Nz_i64>,i64>(None)};
const SOME_BITS_i64:i64=unsafe{intrinsics::transmute::<Option<Nz_i64>,i64>(Some(Nz_i64::new_unchecked(1i64)))};
const MINUS_BITS_i64:i64=unsafe{intrinsics::transmute::<Option<Nz_i64>,i64>(Some(Nz_i64::new_unchecked(-1i64)))};
const FROM_BITS_i64:Option<Nz_i64>=unsafe{intrinsics::transmute::<i64,Option<Nz_i64>>(-9223372036854775808i64)};
#[unsafe(no_mangle)]pub extern "C" fn echo_i64(x:Option<Nz_i64>)->Option<Nz_i64>{x}
#[unsafe(no_mangle)]pub extern "C" fn make_i64(x:i64)->Option<Nz_i64>{Nz_i64::new(x)}
#[unsafe(no_mangle)]pub extern "C" fn get_i64(x:Option<Nz_i64>)->i64{match x{Some(x)=>x.as_inner(),None=>0}}
#[unsafe(no_mangle)]pub extern "C" fn invoke_i64(x:Option<Nz_i64>)->Option<Nz_i64>{let f:extern "C" fn(Option<Nz_i64>)->Option<Nz_i64>=echo_i64;f(x)}
#[unsafe(no_mangle)]pub extern "C" fn size_i64()->usize{const{size_of::<Option<Nz_i64>>()}}
#[unsafe(no_mangle)]pub extern "C" fn align_i64()->usize{const{align_of::<Option<Nz_i64>>()}}
#[unsafe(no_mangle)]pub extern "C" fn constant_none_i64()->i64{NONE_BITS_i64}
#[unsafe(no_mangle)]pub extern "C" fn constant_some_i64()->i64{SOME_BITS_i64}
#[unsafe(no_mangle)]pub extern "C" fn constant_minus_i64()->i64{MINUS_BITS_i64}
#[unsafe(no_mangle)]pub extern "C" fn constant_round_i64()->i64{get_i64(FROM_BITS_i64)}

#[derive(crate::clone::Clone,crate::marker::Copy)]#[repr(transparent)]pub struct Nz_i128(pattern_type!(i128 is ..=-1|1..));
impl Nz_i128{
 pub const fn new(val:i128)->Option<Self>{#[allow(non_contiguous_range_endpoints)]if let ..=-1|1.. = val{Some(unsafe{Nz_i128(intrinsics::transmute(val))})}else{None}}
 pub const unsafe fn new_unchecked(val:i128)->Self{unsafe{intrinsics::transmute(val)}}
 pub const fn as_inner(self)->i128{unsafe{intrinsics::transmute(self)}}
}
const NONE_BITS_i128:i128=unsafe{intrinsics::transmute::<Option<Nz_i128>,i128>(None)};
const SOME_BITS_i128:i128=unsafe{intrinsics::transmute::<Option<Nz_i128>,i128>(Some(Nz_i128::new_unchecked(1i128)))};
const MINUS_BITS_i128:i128=unsafe{intrinsics::transmute::<Option<Nz_i128>,i128>(Some(Nz_i128::new_unchecked(-1i128)))};
const FROM_BITS_i128:Option<Nz_i128>=unsafe{intrinsics::transmute::<i128,Option<Nz_i128>>(-170141183460469231731687303715884105728i128)};
#[unsafe(no_mangle)]pub extern "C" fn echo_i128(x:Option<Nz_i128>)->Option<Nz_i128>{x}
#[unsafe(no_mangle)]pub extern "C" fn make_i128(x:i128)->Option<Nz_i128>{Nz_i128::new(x)}
#[unsafe(no_mangle)]pub extern "C" fn get_i128(x:Option<Nz_i128>)->i128{match x{Some(x)=>x.as_inner(),None=>0}}
#[unsafe(no_mangle)]pub extern "C" fn invoke_i128(x:Option<Nz_i128>)->Option<Nz_i128>{let f:extern "C" fn(Option<Nz_i128>)->Option<Nz_i128>=echo_i128;f(x)}
#[unsafe(no_mangle)]pub extern "C" fn size_i128()->usize{const{size_of::<Option<Nz_i128>>()}}
#[unsafe(no_mangle)]pub extern "C" fn align_i128()->usize{const{align_of::<Option<Nz_i128>>()}}
#[unsafe(no_mangle)]pub extern "C" fn constant_none_i128()->i128{NONE_BITS_i128}
#[unsafe(no_mangle)]pub extern "C" fn constant_some_i128()->i128{SOME_BITS_i128}
#[unsafe(no_mangle)]pub extern "C" fn constant_minus_i128()->i128{MINUS_BITS_i128}
#[unsafe(no_mangle)]pub extern "C" fn constant_round_i128()->i128{get_i128(FROM_BITS_i128)}

#[derive(crate::clone::Clone,crate::marker::Copy)]#[repr(transparent)]pub struct Nz_isize(pattern_type!(isize is ..=-1|1..));
impl Nz_isize{
 pub const fn new(val:isize)->Option<Self>{#[allow(non_contiguous_range_endpoints)]if let ..=-1|1.. = val{Some(unsafe{Nz_isize(intrinsics::transmute(val))})}else{None}}
 pub const unsafe fn new_unchecked(val:isize)->Self{unsafe{intrinsics::transmute(val)}}
 pub const fn as_inner(self)->isize{unsafe{intrinsics::transmute(self)}}
}
const NONE_BITS_isize:isize=unsafe{intrinsics::transmute::<Option<Nz_isize>,isize>(None)};
const SOME_BITS_isize:isize=unsafe{intrinsics::transmute::<Option<Nz_isize>,isize>(Some(Nz_isize::new_unchecked(1isize)))};
const MINUS_BITS_isize:isize=unsafe{intrinsics::transmute::<Option<Nz_isize>,isize>(Some(Nz_isize::new_unchecked(-1isize)))};
const FROM_BITS_isize:Option<Nz_isize>=unsafe{intrinsics::transmute::<isize,Option<Nz_isize>>(-9223372036854775808isize)};
#[unsafe(no_mangle)]pub extern "C" fn echo_isize(x:Option<Nz_isize>)->Option<Nz_isize>{x}
#[unsafe(no_mangle)]pub extern "C" fn make_isize(x:isize)->Option<Nz_isize>{Nz_isize::new(x)}
#[unsafe(no_mangle)]pub extern "C" fn get_isize(x:Option<Nz_isize>)->isize{match x{Some(x)=>x.as_inner(),None=>0}}
#[unsafe(no_mangle)]pub extern "C" fn invoke_isize(x:Option<Nz_isize>)->Option<Nz_isize>{let f:extern "C" fn(Option<Nz_isize>)->Option<Nz_isize>=echo_isize;f(x)}
#[unsafe(no_mangle)]pub extern "C" fn size_isize()->usize{const{size_of::<Option<Nz_isize>>()}}
#[unsafe(no_mangle)]pub extern "C" fn align_isize()->usize{const{align_of::<Option<Nz_isize>>()}}
#[unsafe(no_mangle)]pub extern "C" fn constant_none_isize()->isize{NONE_BITS_isize}
#[unsafe(no_mangle)]pub extern "C" fn constant_some_isize()->isize{SOME_BITS_isize}
#[unsafe(no_mangle)]pub extern "C" fn constant_minus_isize()->isize{MINUS_BITS_isize}
#[unsafe(no_mangle)]pub extern "C" fn constant_round_isize()->isize{get_isize(FROM_BITS_isize)}

#[repr(transparent)]pub struct Outer(Nz_i8);
#[unsafe(no_mangle)]pub extern "C" fn nested(x:Option<Outer>)->Option<Outer>{x}
#[repr(C)]pub struct Pair{pub a:u64,pub b:u64}
#[unsafe(no_mangle)]pub extern "C" fn mixed(x:Option<Nz_i8>,p:Pair)->Option<Nz_i8>{let _=p;x}
#[repr(C)]pub struct Packet{pub value:Option<Nz_i8>,pub stamp:u32}
#[unsafe(no_mangle)]pub extern "C" fn packet(x:Packet)->Packet{x}

const BAD:Option<Nz_i8>=Some(unsafe{Nz_i8::new_unchecked(0i8)});

#[unsafe(no_mangle)]pub extern "C" fn reached_bad()->i8{get_i8(BAD)}
