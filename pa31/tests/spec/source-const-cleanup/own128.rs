#![feature(impl_restriction)]
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


impl Clone for i32{fn clone(&self)->Self{*self}}impl Copy for i32{}

#[lang="drop"]#[stable(feature="rust1",since="1.0.0")]#[rustc_const_unstable(feature="const_destruct",issue="133214")]pub const trait Drop{#[stable(feature="rust1",since="1.0.0")]fn drop(&mut self);}
pub struct N;impl Drop for N{fn drop(&mut self){}}
pub struct C;#[rustc_const_unstable(feature="const_destruct",issue="133214")]const impl Drop for C{fn drop(&mut self){}}
pub struct S<T:Destruct>{head:T,f0:T,f1:T,f2:T,f3:T,f4:T,f5:T,f6:T,f7:T,f8:T,f9:T,f10:T,f11:T,f12:T,f13:T,f14:T,f15:T,f16:T,f17:T,f18:T,f19:T,f20:T,f21:T,f22:T,f23:T,f24:T,f25:T,f26:T,f27:T,f28:T,f29:T,f30:T,f31:T,f32:T,f33:T,f34:T,f35:T,f36:T,f37:T,f38:T,f39:T,f40:T,f41:T,f42:T,f43:T,f44:T,f45:T,f46:T,f47:T,f48:T,f49:T,f50:T,f51:T,f52:T,f53:T,f54:T,f55:T,f56:T,f57:T,f58:T,f59:T,f60:T,f61:T,f62:T,f63:T,f64:T,f65:T,f66:T,f67:T,f68:T,f69:T,f70:T,f71:T,f72:T,f73:T,f74:T,f75:T,f76:T,f77:T,f78:T,f79:T,f80:T,f81:T,f82:T,f83:T,f84:T,f85:T,f86:T,f87:T,f88:T,f89:T,f90:T,f91:T,f92:T,f93:T,f94:T,f95:T,f96:T,f97:T,f98:T,f99:T,f100:T,f101:T,f102:T,f103:T,f104:T,f105:T,f106:T,f107:T,f108:T,f109:T,f110:T,f111:T,f112:T,f113:T,f114:T,f115:T,f116:T,f117:T,f118:T,f119:T,f120:T,f121:T,f122:T,f123:T,f124:T,f125:T,f126:T,f127:T}
#[rustc_const_unstable(feature="const_destruct",issue="133214")]const impl<T:[const]Destruct> Drop for S<T>{fn drop(&mut self){}}
