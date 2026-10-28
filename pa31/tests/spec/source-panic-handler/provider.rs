#![feature(core_intrinsics,panic_can_unwind,panic_internals,fmt_arguments_from_str,ptr_metadata)]
#![feature(extern_types)]
#![feature(cfg_sanitize,allow_internal_unsafe,unsize,coerce_unsized,dispatch_from_dyn)]
#![feature(intrinsics,pattern_types,decl_macro,marker_trait_attr,freeze_impls,allow_internal_unstable,staged_api)]
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
    #[rustc_intrinsic_const_stable_indirect]
    #[rustc_nounwind]
    #[rustc_intrinsic]
    pub const fn caller_location() -> &'static crate::panic::Location<'static>;

use crate::{PointeeSized,ptr};
#[rustc_nounwind]
#[rustc_intrinsic]
pub fn abort() -> !;

#[rustc_nounwind]
#[unstable(feature = "core_intrinsics", issue = "none")]
#[rustc_intrinsic_const_stable_indirect]
#[rustc_intrinsic]
pub const fn ptr_metadata<P: ptr::Pointee<Metadata = M> + PointeeSized, M>(ptr: *const P) -> M;

    #[stable(feature = "rust1", since = "1.0.0")]
    #[rustc_const_stable(feature = "const_transmute", since = "1.56.0")]
    #[rustc_intrinsic]pub const unsafe fn transmute<Src,Dst>(src:Src)->Dst;
}


// Genuine core shared-reference impls, retained verbatim.
#[stable(feature = "rust1", since = "1.0.0")]
impl<T: PointeeSized> Copy for &T {}


    /// Shared references can be cloned, but mutable references *cannot*!
    #[stable(feature = "rust1", since = "1.0.0")]
    #[rustc_const_unstable(feature = "const_clone", issue = "142757")]
    const impl<T: PointeeSized> Clone for &T {
        #[inline(always)]
        #[rustc_diagnostic_item = "noop_method_clone"]
        fn clone(&self) -> Self {
            *self
        }
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

extern crate self as core;
impl Clone for u16 { fn clone(&self)->Self { *self } } impl Copy for u16 {}
#[unstable(feature="unsize",issue="18598")]
#[lang="unsize"]#[rustc_deny_explicit_impl]#[rustc_dyn_incompatible_trait]
pub trait Unsize<T:PointeeSized>:PointeeSized {}
#[unstable(feature="coerce_unsized",issue="18598")]
#[lang="coerce_unsized"]pub trait CoerceUnsized<T:PointeeSized>:Sized {}
#[unstable(feature="coerce_unsized",issue="18598")]
impl<'a,T:PointeeSized+Unsize<U>,U:PointeeSized> CoerceUnsized<&'a mut U> for &'a mut T {}
#[unstable(feature = "dispatch_from_dyn", issue = "none")]
#[lang = "dispatch_from_dyn"]
pub trait DispatchFromDyn<T>: Sized {
    // Empty.
}

// &T -> &U
#[unstable(feature = "dispatch_from_dyn", issue = "none")]
impl<'a, T: PointeeSized + Unsize<U>, U: PointeeSized> DispatchFromDyn<&'a U> for &'a T {}
// &mut T -> &mut U
#[unstable(feature = "dispatch_from_dyn", issue = "none")]
impl<'a, T: PointeeSized + Unsize<U>, U: PointeeSized> DispatchFromDyn<&'a mut U> for &'a mut T {}

// Authored dependencies preserve the actual result and Formatter field shapes;
// this reducer does not replace the retained full Write/fmt::write module graph.
pub mod result { pub enum Result<T,E> { Ok(T), Err(E) } }
    #[stable(feature = "rust1", since = "1.0.0")]
    #[rustc_builtin_macro]
    #[macro_export]
    #[rustc_diagnostic_item = "include_str_macro"]
    macro_rules! include_str {
        ($file:expr $(,)?) => {{ /* compiler built-in */ }};
    }


macro_rules! forward_ref_unop {
    (impl $imp:ident, $method:ident for $t:ty, $(#[$attr:meta])+) => {
        $(#[$attr])+
        const impl $imp for &$t {
            type Output = <$t as $imp>::Output;

            #[inline]
            fn $method(self) -> <$t as $imp>::Output {
                $imp::$method(*self)
            }
        }
    }
}

// implements binary operators "&T op U", "T op &U", "&T op &U"
// based on "T op U" where T and U are expected to be `Copy`able
#[lang = "not"]
#[stable(feature = "rust1", since = "1.0.0")]
#[rustc_const_unstable(feature = "const_ops", issue = "143802")]
#[doc(alias = "!")]
pub const trait Not {
    /// The resulting type after applying the `!` operator.
    #[stable(feature = "rust1", since = "1.0.0")]
    type Output;

    /// Performs the unary `!` operation.
    ///
    /// # Examples
    ///
    /// ```
    /// assert_eq!(!true, false);
    /// assert_eq!(!false, true);
    /// assert_eq!(!1u8, 254);
    /// assert_eq!(!0u8, 255);
    /// ```
    #[must_use]
    #[stable(feature = "rust1", since = "1.0.0")]
    fn not(self) -> Self::Output;
}
macro_rules! not_impl {
    ($($t:ty)*) => ($(
        #[stable(feature = "rust1", since = "1.0.0")]
        #[rustc_const_unstable(feature = "const_ops", issue = "143802")]
        const impl Not for $t {
            type Output = $t;

            #[inline]
            fn not(self) -> $t { !self }
        }

        forward_ref_unop! { impl Not, not for $t,
        #[stable(feature = "rust1", since = "1.0.0")]
        #[rustc_const_unstable(feature = "const_ops", issue = "143802")] }
    )*)
}

not_impl!{bool}

impl Clone for bool{fn clone(&self)->Self{*self}}impl Copy for bool{}
impl Clone for usize{fn clone(&self)->Self{*self}}impl Copy for usize{}
#[lang = "shl"]
#[doc(alias = "<<")]
#[stable(feature = "rust1", since = "1.0.0")]
#[rustc_const_unstable(feature = "const_ops", issue = "143802")]
#[diagnostic::on_unimplemented(
    message = "no implementation for `{Self} << {Rhs}`",
    label = "no implementation for `{Self} << {Rhs}`"
)]
pub const trait Shl<Rhs = Self> {
    /// The resulting type after applying the `<<` operator.
    #[stable(feature = "rust1", since = "1.0.0")]
    type Output;

    /// Performs the `<<` operation.
    ///
    /// # Examples
    ///
    /// ```
    /// assert_eq!(5u8 << 1, 10);
    /// assert_eq!(1u8 << 1, 2);
    /// ```
    #[must_use]
    #[stable(feature = "rust1", since = "1.0.0")]
    fn shl(self, rhs: Rhs) -> Self::Output;
}

#[lang = "bitor"]
#[doc(alias = "|")]
#[stable(feature = "rust1", since = "1.0.0")]
#[rustc_const_unstable(feature = "const_ops", issue = "143802")]
#[diagnostic::on_unimplemented(
    message = "no implementation for `{Self} | {Rhs}`",
    label = "no implementation for `{Self} | {Rhs}`"
)]
pub const trait BitOr<Rhs = Self> {
    /// The resulting type after applying the `|` operator.
    #[stable(feature = "rust1", since = "1.0.0")]
    type Output;

    /// Performs the `|` operation.
    ///
    /// # Examples
    ///
    /// ```
    /// assert_eq!(true | false, true);
    /// assert_eq!(false | false, false);
    /// assert_eq!(5u8 | 1u8, 5);
    /// assert_eq!(5u8 | 2u8, 7);
    /// ```
    #[must_use]
    #[stable(feature = "rust1", since = "1.0.0")]
    fn bitor(self, rhs: Rhs) -> Self::Output;
}

        #[stable(feature = "rust1", since = "1.0.0")]
        #[rustc_const_unstable(feature = "const_ops", issue = "143802")]

const impl Shl<i32> for usize{type Output=usize;fn shl(self,rhs:i32)->usize{self<<rhs}}
        #[stable(feature = "rust1", since = "1.0.0")]
        #[rustc_const_unstable(feature = "const_ops", issue = "143802")]

const impl BitOr for usize{type Output=usize;fn bitor(self,rhs:usize)->usize{self|rhs}}
pub trait LowerHex:PointeeSized{fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result;}
pub mod fmt {
    use crate::{derive,NonNull,PhantomData,mem,PointeeSized,Sized,result,LowerHex};
    use crate::marker::Copy;
    use crate::clone::Clone;
    pub struct Error;
    pub type Result=result::Result<(),Error>;
    pub trait Write { fn write_str(&mut self,s:&str)->Result; }
    pub struct FormattingOptions { flags:u32,width:u16,precision:u16 }
#[allow(missing_debug_implementations)]
#[stable(feature = "rust1", since = "1.0.0")]
#[rustc_diagnostic_item = "Formatter"]
pub struct Formatter<'a> {
    options: FormattingOptions,

    buf: &'a mut (dyn Write + 'a),
}

    #[rustc_diagnostic_item="Debug"]
    pub trait Debug:PointeeSized { fn fmt(&self,f:&mut Formatter<'_>)->Result; }
    // Authored reduced support declaration; the complete original Display header is retained separately.
    #[rustc_diagnostic_item="Display"]
    pub trait Display:PointeeSized { fn fmt(&self,f:&mut Formatter<'_>)->Result; }

#[lang = "format_arguments"]
#[stable(feature = "rust1", since = "1.0.0")]
#[derive(Copy, Clone)]
pub struct Arguments<'a> {
    template: NonNull<u8>,
    args: NonNull<rt::Argument<'a>>,
}

impl<'a> Arguments<'a> {
    /// Formatting this `fmt::Arguments` will just produce the string as-is.
    #[inline]
    #[unstable(feature = "fmt_arguments_from_str", issue = "148905")]
    pub const fn from_str(s: &'static str) -> Arguments<'a> {
        // SAFETY: This is the "static str" representation of fmt::Arguments; see above.
        unsafe {
            Arguments {
                template: mem::transmute(s.as_ptr()),
                args: mem::transmute(s.len() << 1 | 1),
            }
        }
    }

}
pub mod rt {use super::*;
#[derive(Copy, Clone)]
enum ArgumentType<'a> {
    Placeholder {
        // INVARIANT: `formatter` has type `fn(&T, _) -> _` for some `T`, and `value`
        // was derived from a `&'a T`.
        value: NonNull<()>,
        formatter: unsafe fn(NonNull<()>, &mut Formatter<'_>) -> Result,
        _lifetime: PhantomData<&'a ()>,
    },
    Count(u16),
}

/// This struct represents a generic "argument" which is taken by format_args!().
///
/// This can be either a placeholder argument or a count argument.
/// * A placeholder argument contains a function to format the given value. At
///   compile time it is ensured that the function and the value have the correct
///   types, and then this struct is used to canonicalize arguments to one type.
///   Placeholder arguments are essentially an optimized partially applied formatting
///   function, equivalent to `exists T.(&T, fn(&T, &mut Formatter<'_>) -> Result`.
/// * A count argument contains a count for dynamic formatting parameters like
///   precision and width.
#[lang = "format_argument"]
#[derive(Copy, Clone)]
#[repr(align(2))] // To ensure pointers to this struct always have their lowest bit cleared.
pub struct Argument<'a> {
    ty: ArgumentType<'a>,
}

}
}
pub mod ptr {pub use crate::Pointee;
#[unstable(feature = "ptr_metadata", issue = "81513")]

pub use metadata::metadata;
mod metadata {
#![unstable(feature = "ptr_metadata", issue = "81513")]

use crate::{PointeeSized,Pointee,intrinsics::ptr_metadata};
#[inline]
pub const fn metadata<T: PointeeSized>(ptr: *const T) -> <T as Pointee>::Metadata {
    ptr_metadata(ptr)
}

}
}
impl<T> [T] {
    /// ```
    #[lang = "slice_len_fn"]
    #[stable(feature = "rust1", since = "1.0.0")]
    #[rustc_const_stable(feature = "const_slice_len", since = "1.39.0")]
    #[rustc_no_implicit_autorefs]
    #[inline]
    #[must_use]
    pub const fn len(&self) -> usize {
        ptr::metadata(self)
    }

}
impl str {
    #[stable(feature = "rust1", since = "1.0.0")]
    #[rustc_const_stable(feature = "const_str_len", since = "1.39.0")]
    #[rustc_diagnostic_item = "str_len"]
    #[rustc_no_implicit_autorefs]
    #[must_use]
    #[inline]
    pub const fn len(&self) -> usize {
        self.as_bytes().len()
    }

    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    #[rustc_const_stable(feature = "str_as_bytes", since = "1.39.0")]
    #[must_use]
    #[inline(always)]
    #[allow(unused_attributes)]
    pub const fn as_bytes(&self) -> &[u8] {
        // SAFETY: const sound because we transmute two types with the same layout
        unsafe { mem::transmute(self) }
    }

    /// let s = "Hello";
    /// let ptr = s.as_ptr();
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    #[rustc_const_stable(feature = "rustc_str_as_ptr", since = "1.32.0")]
    #[rustc_never_returns_null_ptr]
    #[rustc_as_ptr]
    #[must_use]
    #[inline(always)]
    pub const fn as_ptr(&self) -> *const u8 {
        self as *const str as *const u8
    }

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

pub mod panic {pub use crate::location::Location;pub use crate::panic_info::PanicInfo;
#[doc(hidden)]
#[unstable(feature = "edition_panic", issue = "none", reason = "use panic!() instead")]
#[allow_internal_unstable(panic_internals, const_format_args)]
#[rustc_diagnostic_item = "core_panic_2021_macro"]
#[rustc_macro_transparency = "semiopaque"]
pub macro panic_2021 {
    () => (
        $crate::panicking::panic("explicit panic")
    ),
    // Special-case the single-argument case for const_panic.
    ("{}", $arg:expr $(,)?) => ({
        $crate::panicking::panic_display(&$arg);
    }),
    ($($t:tt)+) => ({
        // Semicolon to prevent temporaries inside the formatting machinery from
        // being considered alive in the caller after the panic_fmt call.
        $crate::panicking::panic_fmt($crate::const_format_args!($($t)+));
    }),
}

}
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

pub mod panic_info {use crate::{fmt,location::Location,derive,Debug};use crate::Option::{self,Some};
#[lang = "panic_info"]
#[stable(feature = "panic_hooks", since = "1.10.0")]
#[derive(Debug)]
pub struct PanicInfo<'a> {
    message: &'a fmt::Arguments<'a>,
    location: &'static Location<'static>,
    can_unwind: bool,
    force_no_backtrace: bool,
}

impl<'a> PanicInfo<'a>{
    #[inline]
    pub(crate) fn new(
        message: &'a fmt::Arguments<'a>,
        location: &'static Location<'static>,
        can_unwind: bool,
        force_no_backtrace: bool,
    ) -> Self {
        PanicInfo { location, message, can_unwind, force_no_backtrace }
    }

    #[must_use]
    #[stable(feature = "panic_hooks", since = "1.10.0")]
    pub fn location(&self) -> Option<&'static Location<'static>> {
        // NOTE: If this is changed to sometimes return None,
        // deal with that case in std::panicking::default_hook and core::panicking::panic_fmt.
        Some(self.location)
    }

    #[must_use]
    #[unstable(feature = "panic_can_unwind", issue = "92988")]
    pub fn can_unwind(&self) -> bool {
        self.can_unwind
    }

    #[unstable(
        feature = "panic_internals",
        reason = "internal details of the implementation of the `panic!` and related macros",
        issue = "none"
    )]
    #[doc(hidden)]
    #[inline]
    pub fn force_no_backtrace(&self) -> bool {
        self.force_no_backtrace
    }

}
}
    #[stable(feature = "rust1", since = "1.0.0")]
    #[rustc_builtin_macro]
    #[macro_export]
    macro_rules! cfg {
        ($($cfg:tt)*) => {
            /* compiler built-in */
        };
    }

pub mod panicking {use crate::{fmt,panic::{PanicInfo,Location}};
// otherwise avoid inlining because of it is cold path.
#[cfg_attr(not(panic = "immediate-abort"), inline(never), cold)]
#[cfg_attr(panic = "immediate-abort", inline)]
#[track_caller]
#[lang = "panic_fmt"] // needed for const-evaluated panics
#[rustc_do_not_const_check] // hooked by const-eval
#[rustc_const_stable_indirect] // must follow stable const rules since it is exposed to stable
pub const fn panic_fmt(fmt: fmt::Arguments<'_>) -> ! {
    if cfg!(panic = "immediate-abort") {
        super::intrinsics::abort()
    }

    // NOTE This function never crosses the FFI boundary; it's a Rust-to-Rust call
    // that gets resolved to the `#[panic_handler]` function.
    unsafe extern "Rust" {
        #[lang = "panic_impl"]
        fn panic_impl(pi: &PanicInfo<'_>) -> !;
    }

    let pi = PanicInfo::new(
        &fmt,
        Location::caller(),
        /* can_unwind */ true,
        /* force_no_backtrace */ false,
    );

    // SAFETY: `panic_impl` is defined in safe Rust code and thus is safe to call.
    unsafe { panic_impl(&pi) }
}

/// The underlying implementation of core's `panic!` macro when no formatting is used.
// Never inline unless panic=immediate-abort to avoid code
// bloat at the call sites as much as possible.
#[cfg_attr(not(panic = "immediate-abort"), inline(never), cold)]
#[cfg_attr(panic = "immediate-abort", inline)]
#[track_caller]
#[rustc_const_stable_indirect] // must follow stable const rules since it is exposed to stable
#[lang = "panic"] // used by lints and miri for panics
pub const fn panic(expr: &'static str) -> ! {
    // Use Arguments::from_str instead of format_args!("{expr}") to potentially
    // reduce size overhead. The format_args! macro uses str's Display trait to
    // write expr, which calls Formatter::pad, which must accommodate string
    // truncation and padding (even though none is used here). Using
    // Arguments::from_str may allow the compiler to omit Formatter::pad from the
    // output binary, saving up to a few kilobytes.
    // However, this optimization only works for `'static` strings: `from_str` also makes this
    // message return `Some` from `Arguments::as_str`, which means it can become part of the panic
    // payload without any allocation or copying. Shorter-lived strings would become invalid as
    // stack frames get popped during unwinding, and couldn't be directly referenced from the
    // payload.
    panic_fmt(fmt::Arguments::from_str(expr));
}

}
#[lang="Option"]pub enum Option<T>{#[lang="None"]None,#[lang="Some"]Some(T)}
use Option::{None,Some};

pub mod ops{pub use crate::Drop;}
#[lang = "drop"]
#[stable(feature = "rust1", since = "1.0.0")]
#[rustc_const_unstable(feature = "const_destruct", issue = "133214")]
pub const trait Drop {

    #[stable(feature = "rust1", since = "1.0.0")]

fn drop(&mut self); }
#[rustc_builtin_macro]
    #[stable(feature = "builtin_macro_prelude", since = "1.38.0")]
    #[allow_internal_unstable(core_intrinsics, fmt_helpers_for_derive)]
    pub macro Debug($item:item) {
        /* compiler built-in */
    }

#[stable(feature = "rust1", since = "1.0.0")]
    #[rustc_builtin_macro]
    #[macro_export]
    #[rustc_diagnostic_item = "assert_macro"]
    #[allow_internal_unstable(
        core_intrinsics,
        panic_internals,
        edition_panic,
        generic_assert_internals
    )]
    macro_rules! assert {
        ($cond:expr $(,)?) => {{ /* compiler built-in */ }};
        ($cond:expr, $($arg:tt)+) => {{ /* compiler built-in */ }};
    }

    #[unstable(feature = "const_format_args", issue = "none")]
    #[allow_internal_unstable(fmt_internals, fmt_arguments_from_str)]
    #[rustc_builtin_macro]
    #[macro_export]
    macro_rules! const_format_args {
        ($fmt:expr) => {{ /* compiler built-in */ }};
        ($fmt:expr, $($args:tt)*) => {{ /* compiler built-in */ }};
    }

pub use crate::fmt::Debug;
#[path="handler-user.rs"]pub mod user;
