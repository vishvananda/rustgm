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
#[rustc_const_unstable(feature="const_destruct",issue="133214")]pub const fn work0(){let _value=C;}
#[rustc_const_unstable(feature="const_destruct",issue="133214")]pub const fn work1(){let _value=C;}
#[rustc_const_unstable(feature="const_destruct",issue="133214")]pub const fn work2(){let _value=C;}
#[rustc_const_unstable(feature="const_destruct",issue="133214")]pub const fn work3(){let _value=C;}
#[rustc_const_unstable(feature="const_destruct",issue="133214")]pub const fn work4(){let _value=C;}
#[rustc_const_unstable(feature="const_destruct",issue="133214")]pub const fn work5(){let _value=C;}
#[rustc_const_unstable(feature="const_destruct",issue="133214")]pub const fn work6(){let _value=C;}
#[rustc_const_unstable(feature="const_destruct",issue="133214")]pub const fn work7(){let _value=C;}
#[rustc_const_unstable(feature="const_destruct",issue="133214")]pub const fn work8(){let _value=C;}
#[rustc_const_unstable(feature="const_destruct",issue="133214")]pub const fn work9(){let _value=C;}
#[rustc_const_unstable(feature="const_destruct",issue="133214")]pub const fn work10(){let _value=C;}
#[rustc_const_unstable(feature="const_destruct",issue="133214")]pub const fn work11(){let _value=C;}
#[rustc_const_unstable(feature="const_destruct",issue="133214")]pub const fn work12(){let _value=C;}
#[rustc_const_unstable(feature="const_destruct",issue="133214")]pub const fn work13(){let _value=C;}
#[rustc_const_unstable(feature="const_destruct",issue="133214")]pub const fn work14(){let _value=C;}
#[rustc_const_unstable(feature="const_destruct",issue="133214")]pub const fn work15(){let _value=C;}
#[rustc_const_unstable(feature="const_destruct",issue="133214")]pub const fn work16(){let _value=C;}
#[rustc_const_unstable(feature="const_destruct",issue="133214")]pub const fn work17(){let _value=C;}
#[rustc_const_unstable(feature="const_destruct",issue="133214")]pub const fn work18(){let _value=C;}
#[rustc_const_unstable(feature="const_destruct",issue="133214")]pub const fn work19(){let _value=C;}
#[rustc_const_unstable(feature="const_destruct",issue="133214")]pub const fn work20(){let _value=C;}
#[rustc_const_unstable(feature="const_destruct",issue="133214")]pub const fn work21(){let _value=C;}
#[rustc_const_unstable(feature="const_destruct",issue="133214")]pub const fn work22(){let _value=C;}
#[rustc_const_unstable(feature="const_destruct",issue="133214")]pub const fn work23(){let _value=C;}
#[rustc_const_unstable(feature="const_destruct",issue="133214")]pub const fn work24(){let _value=C;}
#[rustc_const_unstable(feature="const_destruct",issue="133214")]pub const fn work25(){let _value=C;}
#[rustc_const_unstable(feature="const_destruct",issue="133214")]pub const fn work26(){let _value=C;}
#[rustc_const_unstable(feature="const_destruct",issue="133214")]pub const fn work27(){let _value=C;}
#[rustc_const_unstable(feature="const_destruct",issue="133214")]pub const fn work28(){let _value=C;}
#[rustc_const_unstable(feature="const_destruct",issue="133214")]pub const fn work29(){let _value=C;}
#[rustc_const_unstable(feature="const_destruct",issue="133214")]pub const fn work30(){let _value=C;}
#[rustc_const_unstable(feature="const_destruct",issue="133214")]pub const fn work31(){let _value=C;}
#[rustc_const_unstable(feature="const_destruct",issue="133214")]pub const fn work32(){let _value=C;}
#[rustc_const_unstable(feature="const_destruct",issue="133214")]pub const fn work33(){let _value=C;}
#[rustc_const_unstable(feature="const_destruct",issue="133214")]pub const fn work34(){let _value=C;}
#[rustc_const_unstable(feature="const_destruct",issue="133214")]pub const fn work35(){let _value=C;}
#[rustc_const_unstable(feature="const_destruct",issue="133214")]pub const fn work36(){let _value=C;}
#[rustc_const_unstable(feature="const_destruct",issue="133214")]pub const fn work37(){let _value=C;}
#[rustc_const_unstable(feature="const_destruct",issue="133214")]pub const fn work38(){let _value=C;}
#[rustc_const_unstable(feature="const_destruct",issue="133214")]pub const fn work39(){let _value=C;}
#[rustc_const_unstable(feature="const_destruct",issue="133214")]pub const fn work40(){let _value=C;}
#[rustc_const_unstable(feature="const_destruct",issue="133214")]pub const fn work41(){let _value=C;}
#[rustc_const_unstable(feature="const_destruct",issue="133214")]pub const fn work42(){let _value=C;}
#[rustc_const_unstable(feature="const_destruct",issue="133214")]pub const fn work43(){let _value=C;}
#[rustc_const_unstable(feature="const_destruct",issue="133214")]pub const fn work44(){let _value=C;}
#[rustc_const_unstable(feature="const_destruct",issue="133214")]pub const fn work45(){let _value=C;}
#[rustc_const_unstable(feature="const_destruct",issue="133214")]pub const fn work46(){let _value=C;}
#[rustc_const_unstable(feature="const_destruct",issue="133214")]pub const fn work47(){let _value=C;}
#[rustc_const_unstable(feature="const_destruct",issue="133214")]pub const fn work48(){let _value=C;}
#[rustc_const_unstable(feature="const_destruct",issue="133214")]pub const fn work49(){let _value=C;}
#[rustc_const_unstable(feature="const_destruct",issue="133214")]pub const fn work50(){let _value=C;}
#[rustc_const_unstable(feature="const_destruct",issue="133214")]pub const fn work51(){let _value=C;}
#[rustc_const_unstable(feature="const_destruct",issue="133214")]pub const fn work52(){let _value=C;}
#[rustc_const_unstable(feature="const_destruct",issue="133214")]pub const fn work53(){let _value=C;}
#[rustc_const_unstable(feature="const_destruct",issue="133214")]pub const fn work54(){let _value=C;}
#[rustc_const_unstable(feature="const_destruct",issue="133214")]pub const fn work55(){let _value=C;}
#[rustc_const_unstable(feature="const_destruct",issue="133214")]pub const fn work56(){let _value=C;}
#[rustc_const_unstable(feature="const_destruct",issue="133214")]pub const fn work57(){let _value=C;}
#[rustc_const_unstable(feature="const_destruct",issue="133214")]pub const fn work58(){let _value=C;}
#[rustc_const_unstable(feature="const_destruct",issue="133214")]pub const fn work59(){let _value=C;}
#[rustc_const_unstable(feature="const_destruct",issue="133214")]pub const fn work60(){let _value=C;}
#[rustc_const_unstable(feature="const_destruct",issue="133214")]pub const fn work61(){let _value=C;}
#[rustc_const_unstable(feature="const_destruct",issue="133214")]pub const fn work62(){let _value=C;}
#[rustc_const_unstable(feature="const_destruct",issue="133214")]pub const fn work63(){let _value=C;}
#[rustc_const_unstable(feature="const_destruct",issue="133214")]pub const fn work64(){let _value=C;}
#[rustc_const_unstable(feature="const_destruct",issue="133214")]pub const fn work65(){let _value=C;}
#[rustc_const_unstable(feature="const_destruct",issue="133214")]pub const fn work66(){let _value=C;}
#[rustc_const_unstable(feature="const_destruct",issue="133214")]pub const fn work67(){let _value=C;}
#[rustc_const_unstable(feature="const_destruct",issue="133214")]pub const fn work68(){let _value=C;}
#[rustc_const_unstable(feature="const_destruct",issue="133214")]pub const fn work69(){let _value=C;}
#[rustc_const_unstable(feature="const_destruct",issue="133214")]pub const fn work70(){let _value=C;}
#[rustc_const_unstable(feature="const_destruct",issue="133214")]pub const fn work71(){let _value=C;}
#[rustc_const_unstable(feature="const_destruct",issue="133214")]pub const fn work72(){let _value=C;}
#[rustc_const_unstable(feature="const_destruct",issue="133214")]pub const fn work73(){let _value=C;}
#[rustc_const_unstable(feature="const_destruct",issue="133214")]pub const fn work74(){let _value=C;}
#[rustc_const_unstable(feature="const_destruct",issue="133214")]pub const fn work75(){let _value=C;}
#[rustc_const_unstable(feature="const_destruct",issue="133214")]pub const fn work76(){let _value=C;}
#[rustc_const_unstable(feature="const_destruct",issue="133214")]pub const fn work77(){let _value=C;}
#[rustc_const_unstable(feature="const_destruct",issue="133214")]pub const fn work78(){let _value=C;}
#[rustc_const_unstable(feature="const_destruct",issue="133214")]pub const fn work79(){let _value=C;}
#[rustc_const_unstable(feature="const_destruct",issue="133214")]pub const fn work80(){let _value=C;}
#[rustc_const_unstable(feature="const_destruct",issue="133214")]pub const fn work81(){let _value=C;}
#[rustc_const_unstable(feature="const_destruct",issue="133214")]pub const fn work82(){let _value=C;}
#[rustc_const_unstable(feature="const_destruct",issue="133214")]pub const fn work83(){let _value=C;}
#[rustc_const_unstable(feature="const_destruct",issue="133214")]pub const fn work84(){let _value=C;}
#[rustc_const_unstable(feature="const_destruct",issue="133214")]pub const fn work85(){let _value=C;}
#[rustc_const_unstable(feature="const_destruct",issue="133214")]pub const fn work86(){let _value=C;}
#[rustc_const_unstable(feature="const_destruct",issue="133214")]pub const fn work87(){let _value=C;}
#[rustc_const_unstable(feature="const_destruct",issue="133214")]pub const fn work88(){let _value=C;}
#[rustc_const_unstable(feature="const_destruct",issue="133214")]pub const fn work89(){let _value=C;}
#[rustc_const_unstable(feature="const_destruct",issue="133214")]pub const fn work90(){let _value=C;}
#[rustc_const_unstable(feature="const_destruct",issue="133214")]pub const fn work91(){let _value=C;}
#[rustc_const_unstable(feature="const_destruct",issue="133214")]pub const fn work92(){let _value=C;}
#[rustc_const_unstable(feature="const_destruct",issue="133214")]pub const fn work93(){let _value=C;}
#[rustc_const_unstable(feature="const_destruct",issue="133214")]pub const fn work94(){let _value=C;}
#[rustc_const_unstable(feature="const_destruct",issue="133214")]pub const fn work95(){let _value=C;}
#[rustc_const_unstable(feature="const_destruct",issue="133214")]pub const fn work96(){let _value=C;}
#[rustc_const_unstable(feature="const_destruct",issue="133214")]pub const fn work97(){let _value=C;}
#[rustc_const_unstable(feature="const_destruct",issue="133214")]pub const fn work98(){let _value=C;}
#[rustc_const_unstable(feature="const_destruct",issue="133214")]pub const fn work99(){let _value=C;}
#[rustc_const_unstable(feature="const_destruct",issue="133214")]pub const fn work100(){let _value=C;}
#[rustc_const_unstable(feature="const_destruct",issue="133214")]pub const fn work101(){let _value=C;}
#[rustc_const_unstable(feature="const_destruct",issue="133214")]pub const fn work102(){let _value=C;}
#[rustc_const_unstable(feature="const_destruct",issue="133214")]pub const fn work103(){let _value=C;}
#[rustc_const_unstable(feature="const_destruct",issue="133214")]pub const fn work104(){let _value=C;}
#[rustc_const_unstable(feature="const_destruct",issue="133214")]pub const fn work105(){let _value=C;}
#[rustc_const_unstable(feature="const_destruct",issue="133214")]pub const fn work106(){let _value=C;}
#[rustc_const_unstable(feature="const_destruct",issue="133214")]pub const fn work107(){let _value=C;}
#[rustc_const_unstable(feature="const_destruct",issue="133214")]pub const fn work108(){let _value=C;}
#[rustc_const_unstable(feature="const_destruct",issue="133214")]pub const fn work109(){let _value=C;}
#[rustc_const_unstable(feature="const_destruct",issue="133214")]pub const fn work110(){let _value=C;}
#[rustc_const_unstable(feature="const_destruct",issue="133214")]pub const fn work111(){let _value=C;}
#[rustc_const_unstable(feature="const_destruct",issue="133214")]pub const fn work112(){let _value=C;}
#[rustc_const_unstable(feature="const_destruct",issue="133214")]pub const fn work113(){let _value=C;}
#[rustc_const_unstable(feature="const_destruct",issue="133214")]pub const fn work114(){let _value=C;}
#[rustc_const_unstable(feature="const_destruct",issue="133214")]pub const fn work115(){let _value=C;}
#[rustc_const_unstable(feature="const_destruct",issue="133214")]pub const fn work116(){let _value=C;}
#[rustc_const_unstable(feature="const_destruct",issue="133214")]pub const fn work117(){let _value=C;}
#[rustc_const_unstable(feature="const_destruct",issue="133214")]pub const fn work118(){let _value=C;}
#[rustc_const_unstable(feature="const_destruct",issue="133214")]pub const fn work119(){let _value=C;}
#[rustc_const_unstable(feature="const_destruct",issue="133214")]pub const fn work120(){let _value=C;}
#[rustc_const_unstable(feature="const_destruct",issue="133214")]pub const fn work121(){let _value=C;}
#[rustc_const_unstable(feature="const_destruct",issue="133214")]pub const fn work122(){let _value=C;}
#[rustc_const_unstable(feature="const_destruct",issue="133214")]pub const fn work123(){let _value=C;}
#[rustc_const_unstable(feature="const_destruct",issue="133214")]pub const fn work124(){let _value=C;}
#[rustc_const_unstable(feature="const_destruct",issue="133214")]pub const fn work125(){let _value=C;}
#[rustc_const_unstable(feature="const_destruct",issue="133214")]pub const fn work126(){let _value=C;}
#[rustc_const_unstable(feature="const_destruct",issue="133214")]pub const fn work127(){let _value=C;}
