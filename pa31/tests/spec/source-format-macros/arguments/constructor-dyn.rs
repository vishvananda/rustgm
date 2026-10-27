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
pub mod fmt {
    use crate::{derive,NonNull,PhantomData,mem,PointeeSized,Sized,result};
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
#[lang = "format_arguments"]
#[stable(feature = "rust1", since = "1.0.0")]
#[derive(Copy, Clone)]
pub struct Arguments<'a> {
    template: NonNull<u8>,
    args: NonNull<rt::Argument<'a>>,
}
/// Used by the format_args!() macro to create a fmt::Arguments object.
#[doc(hidden)]
#[rustc_diagnostic_item = "FmtArgumentsNew"]
#[unstable(feature = "fmt_internals", issue = "none")]
impl<'a> Arguments<'a> {
    // SAFETY: The caller must ensure that the provided template and args encode a valid
    // fmt::Arguments, as documented above.
    #[inline]
    pub unsafe fn new<const N: usize, const M: usize>(
        template: &'a [u8; N],
        args: &'a [rt::Argument<'a>; M],
    ) -> Arguments<'a> {
        // SAFETY: Responsibility of the caller.
        unsafe { Arguments { template: mem::transmute(template), args: mem::transmute(args) } }
    }

}
pub mod rt { use super::*;

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
macro_rules! argument_new {
    ($t:ty, $x:expr, $f:expr) => {
        Argument {
            // INVARIANT: this creates an `ArgumentType<'a>` from a `&'a T` and
            // a `fn(&T, ...)`, so the invariant is maintained.
            ty: ArgumentType::Placeholder {
                value: NonNull::<$t>::from_ref($x).cast(),
                // The Rust ABI considers all pointers to be equivalent, so transmuting a fn(&T) to
                // fn(NonNull<()>) and calling it with a NonNull<()> that points at a T is allowed.
                // However, the CFI sanitizer does not allow this, and triggers a crash when it
                // happens.
                //
                // To avoid this crash, we use a helper function when CFI is enabled. To avoid the
                // cost of this helper function (mainly code-size) when it is not needed, we
                // transmute the function pointer otherwise.
                //
                // This is similar to what the Rust compiler does internally with vtables when KCFI
                // is enabled, where it generates trampoline functions that only serve to adjust the
                // expected type of the argument. `ArgumentType::Placeholder` is a bit like a
                // manually constructed trait object, so it is not surprising that the same approach
                // has to be applied here as well.
                //
                // It is still considered problematic (from the Rust side) that CFI rejects entirely
                // legal Rust programs, so we do not consider anything done here a stable guarantee,
                // but meanwhile we carry this work-around to keep Rust compatible with CFI and
                // KCFI.
                #[cfg(not(any(sanitize = "cfi", sanitize = "kcfi")))]
                formatter: {
                    let f: fn(&$t, &mut Formatter<'_>) -> Result = $f;
                    // SAFETY: This is only called with `value`, which has the right type.
                    unsafe { core::mem::transmute(f) }
                },
                #[cfg(any(sanitize = "cfi", sanitize = "kcfi"))]
                formatter: |ptr: NonNull<()>, fmt: &mut Formatter<'_>| {
                    let func = $f;
                    // SAFETY: This is the same type as the `value` field.
                    let r = unsafe { ptr.cast::<$t>().as_ref() };
                    (func)(r, fmt)
                },
                _lifetime: PhantomData,
            },
        }
    };
}
impl Argument<'_> {
    #[inline]
    pub const fn new_debug<T: Debug>(x: &T) -> Argument<'_> {
        argument_new!(T, x, <T as Debug>::fmt)
    }
}
    // Observe genuine Argument storage without substituting a formatting algorithm.
    pub fn observed(args:Arguments<'_>)->(u32,u32) {
        unsafe { crate::template_seen(mem::transmute(args.template)); }
        let pointer:*const [Argument<'_>;2]=unsafe { mem::transmute(args.args) };
        let [first,second]=unsafe { &*pointer };
        (read(first),read(second))
    }
    fn read(arg:&Argument<'_>)->u32 {
        match arg.ty {
            ArgumentType::Placeholder { value,.. }=>unsafe { value.cast::<crate::Probe>().as_ref().0 },
            ArgumentType::Count(_)=>0,
        }
    }
}}
    #[stable(feature = "rust1", since = "1.0.0")]
    #[rustc_diagnostic_item = "format_args_macro"]
    #[allow_internal_unsafe]
    #[allow_internal_unstable(fmt_internals, fmt_arguments_from_str)]
    #[rustc_builtin_macro]
    #[macro_export]
    macro_rules! format_args {
        ($fmt:expr) => {{ /* compiler built-in */ }};
        ($fmt:expr, $($args:tt)*) => {{ /* compiler built-in */ }};
    }

#[rustc_const_unstable(feature="const_destruct",issue="133214")]
#[lang="drop"] pub const trait Drop { fn drop(&mut self); }
unsafe extern "C" {
    fn tick(which:u32)->u32;
    fn gone(value:u32);
    fn read_seen(first:u32,second:u32);
    fn template_seen(bytes:*const u8);
}
pub struct Probe(u32);
impl fmt::Debug for Probe { fn fmt(&self,_:&mut fmt::Formatter<'_>)->fmt::Result { result::Result::Ok(()) } }
impl Drop for Probe { fn drop(&mut self) { unsafe { gone(self.0); } } }
fn first()->Probe { Probe(unsafe {tick(1)}) }
fn second()->Probe { Probe(unsafe {tick(2)}) }
fn construction()->(u32,u32) {
    let args=format_args!("{:?} (1 << {:?})",first(),second());
    let values=fmt::rt::observed(args);
    unsafe { read_seen(values.0,values.1); }
    values
}
#[unsafe(no_mangle)]pub extern "C" fn answer(first:&mut u32)->u32 {
    let values=construction(); *first=values.0; values.1
}

}

unsafe extern "C" {
    /// Opaque type for accessing vtables.
    ///
    /// Private implementation detail of `DynMetadata::size_of` etc.
    /// There is conceptually not actually any Abstract Machine memory behind this pointer.
    type VTable;
}

impl<Dyn: PointeeSized> DynMetadata<Dyn> {
    /// When `DynMetadata` appears as the metadata field of a wide pointer, the rustc_middle layout
    /// computation does magic and the resulting layout is *not* a `FieldsShape::Aggregate`, instead
    /// it is a `FieldsShape::Primitive`. This means that the same type can have different layout
    /// depending on whether it appears as the metadata field of a wide pointer or as a stand-alone
