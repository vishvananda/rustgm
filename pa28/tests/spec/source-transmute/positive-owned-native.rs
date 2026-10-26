#![feature(intrinsics)]#![feature(no_core,lang_items,rustc_attrs,const_trait_impl,const_destruct,fundamental,auto_traits)]
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
#[lang = "destruct"]
#[diagnostic::on_unimplemented(message = "can't drop `{Self}`")]
#[rustc_deny_explicit_impl]
#[rustc_dyn_incompatible_trait]
pub const trait Destruct: PointeeSized {}
#[lang = "freeze"]
pub unsafe auto trait Freeze {}
#[lang = "clone"]
#[rustc_diagnostic_item = "Clone"]
#[rustc_trivial_field_reads]
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
    #[lang = "clone_fn"]
    fn clone(&self) -> Self;

    /// Performs copy-assignment from `source`.
    ///
    /// `a.clone_from(&b)` is equivalent to `a = b.clone()` in functionality,
    /// but can be overridden to reuse the resources of `a` to avoid unnecessary
    /// allocations.
    #[inline]
    fn clone_from(&mut self, source: &Self)
    where
        Self: [const] Destruct,
    {
        *self = source.clone()
    }
}
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

pub mod intrinsics{#[rustc_intrinsic]pub const unsafe fn transmute<Src,Dst>(src:Src)->Dst;}

#[lang="drop"]pub const trait Drop{fn drop(&mut self);}
unsafe extern "C"{fn source_dropped();fn destination_dropped();}
#[repr(C)]struct Src(u8);
#[repr(C)]struct Dst(u8);
impl Drop for Src{fn drop(&mut self){unsafe{source_dropped();}}}
impl Drop for Dst{fn drop(&mut self){unsafe{destination_dropped();}}}
const BITS:u32=unsafe{intrinsics::transmute::<f32,u32>(1.0f32)};
const TAG:usize=unsafe{intrinsics::transmute::<*const u8,usize>(intrinsics::transmute::<usize,*const u8>(13usize))};
#[unsafe(no_mangle)]pub extern "C" fn float_bits(x:f32)->u32{unsafe{intrinsics::transmute(x)}}
#[unsafe(no_mangle)]pub extern "C" fn signed_bits(x:i32)->u32{unsafe{intrinsics::transmute(x)}}
#[unsafe(no_mangle)]pub extern "C" fn const_bits()->u32{BITS}
#[unsafe(no_mangle)]pub extern "C" fn tagged_numeric()->usize{TAG}
#[unsafe(no_mangle)]pub extern "C" fn pointer_roundtrip()->u8{
 let value=9u8;let raw:*const u8=unsafe{intrinsics::transmute(&value)};unsafe{*raw}
}
fn value()->u8{7}
#[unsafe(no_mangle)]pub extern "C" fn function_roundtrip()->u8{
 let original:fn()->u8=value;
 let raw:*const ()=unsafe{intrinsics::transmute(original)};
 let callable:fn()->u8=unsafe{intrinsics::transmute(raw)};callable()
}
#[unsafe(no_mangle)]pub extern "C" fn moved(x:u8)->u8{
 let result:Dst=unsafe{intrinsics::transmute(Src(x))};result.0
}
mod ordinary{pub fn transmute(_:u8)->u8{7}}
#[unsafe(no_mangle)]pub extern "C" fn same_name()->u8{ordinary::transmute(1)}
pub unsafe fn generic_identity<T>(x:T)->T{unsafe{intrinsics::transmute(x)}}
pub unsafe fn generic_thin<T,U>(p:*const T)->*const U{unsafe{intrinsics::transmute(p)}}
#[unsafe(no_mangle)]pub extern "C" fn identity_value()->u8{unsafe{generic_identity(21u8)}}
