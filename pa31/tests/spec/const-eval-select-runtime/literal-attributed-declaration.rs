#![feature(intrinsics,unboxed_closures,fn_traits,tuple_trait,core_intrinsics,const_eval_select)]
#![feature(no_core,lang_items,rustc_attrs,const_trait_impl,fundamental,auto_traits,staged_api,const_clone,const_destruct,sized_hierarchy)]
#![no_core]
#![allow(internal_features,dead_code,unused_variables)]
#![stable(feature="source_headers",since="1.0.0")]
#[unstable(feature = "sized_hierarchy", issue = "144404")]
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
#[unstable(feature = "sized_hierarchy", issue = "144404")]
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
#[stable(feature = "rust1", since = "1.0.0")]
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
#[unstable(feature = "legacy_receiver_trait", issue = "none")]
#[doc(hidden)]
pub trait LegacyReceiver: PointeeSized {
    // Empty.
}

#[unstable(feature = "legacy_receiver_trait", issue = "none")]
impl<T: PointeeSized> LegacyReceiver for &T {}

#[unstable(feature = "legacy_receiver_trait", issue = "none")]
impl<T: PointeeSized> LegacyReceiver for &mut T {}
#[unstable(feature = "const_destruct", issue = "133214")]
#[rustc_const_unstable(feature = "const_destruct", issue = "133214")]
#[lang = "destruct"]
#[diagnostic::on_unimplemented(message = "can't drop `{Self}`")]
#[rustc_deny_explicit_impl]
#[rustc_dyn_incompatible_trait]
pub const trait Destruct: PointeeSized {}
#[lang = "freeze"]
#[unstable(feature = "freeze", issue = "121675")]
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
    #[stable(feature = "rust1", since = "1.0.0")]
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
#[stable(feature="source_headers",since="1.0.0")]
impl Clone for u8 { fn clone(&self)->Self { *self } }
#[stable(feature="source_headers",since="1.0.0")]
impl Copy for u8 {}
#[stable(feature = "convert_id", since = "1.33.0")]
#[rustc_const_stable(feature = "const_identity", since = "1.33.0")]
#[inline(always)]
#[rustc_diagnostic_item = "convert_identity"]
pub const fn identity<T>(x: T) -> T {
    x
}
#[unstable(feature = "tuple_trait", issue = "157987")]
#[lang = "tuple_trait"]
#[diagnostic::on_unimplemented(message = "`{Self}` is not a tuple")]
#[rustc_deny_explicit_impl]
#[rustc_dyn_incompatible_trait]
pub trait Tuple {}
#[lang = "fn_once"]
#[stable(feature = "rust1", since = "1.0.0")]
#[rustc_paren_sugar]
#[rustc_on_unimplemented(
    on(
        Args = "()",
        note = "wrap the `{Self}` in a closure with no arguments: `|| {{ /* code */ }}`"
    ),
    on(
        Self = "unsafe fn",
        note = "unsafe function cannot be called generically without an unsafe block",
        // SAFETY: tidy is not smart enough to tell that the below unsafe block is a string
        label = "call the function in a closure: `|| unsafe {{ /* code */ }}`"
    ),
    message = "expected an `{This:resolved}` closure, found `{Self}`",
    label = "expected an `{This:resolved}` closure, found `{Self}`"
)]
#[fundamental] // so that regex can rely that `&str: !FnMut`
#[must_use = "closures are lazy and do nothing unless called"]
#[rustc_const_unstable(feature = "const_trait_impl", issue = "143874")]
pub const trait FnOnce<Args: Tuple> {
    /// The returned type after the call operator is used.
    #[lang = "fn_once_output"]
    #[stable(feature = "fn_once_output", since = "1.12.0")]
    type Output;

    /// Performs the call operation.
    #[unstable(feature = "fn_traits", issue = "29625")]
    extern "rust-call" fn call_once(self, args: Args) -> Self::Output;
}
#[lang = "fn_mut"]
#[stable(feature = "rust1", since = "1.0.0")]
#[rustc_paren_sugar]
#[rustc_on_unimplemented(
    on(
        Args = "()",
        note = "wrap the `{Self}` in a closure with no arguments: `|| {{ /* code */ }}`"
    ),
    on(
        Self = "unsafe fn",
        note = "unsafe function cannot be called generically without an unsafe block",
        // SAFETY: tidy is not smart enough to tell that the below unsafe block is a string
        label = "call the function in a closure: `|| unsafe {{ /* code */ }}`"
    ),
    message = "expected an `{This:resolved}` closure, found `{Self}`",
    label = "expected an `{This:resolved}` closure, found `{Self}`"
)]
#[fundamental] // so that regex can rely that `&str: !FnMut`
#[must_use = "closures are lazy and do nothing unless called"]
#[rustc_const_unstable(feature = "const_trait_impl", issue = "143874")]
pub const trait FnMut<Args: Tuple>: FnOnce<Args> {
    /// Performs the call operation.
    #[unstable(feature = "fn_traits", issue = "29625")]
    extern "rust-call" fn call_mut(&mut self, args: Args) -> Self::Output;
}
#[lang = "fn"]
#[stable(feature = "rust1", since = "1.0.0")]
#[rustc_paren_sugar]
#[rustc_on_unimplemented(
    on(
        Args = "()",
        note = "wrap the `{Self}` in a closure with no arguments: `|| {{ /* code */ }}`"
    ),
    on(
        Self = "unsafe fn",
        note = "unsafe function cannot be called generically without an unsafe block",
        // SAFETY: tidy is not smart enough to tell that the below unsafe block is a string
        label = "call the function in a closure: `|| unsafe {{ /* code */ }}`"
    ),
    message = "expected an `{This:resolved}` closure, found `{Self}`",
    label = "expected an `{This:resolved}` closure, found `{Self}`"
)]
#[fundamental] // so that regex can rely that `&str: !FnMut`
#[must_use = "closures are lazy and do nothing unless called"]
#[rustc_const_unstable(feature = "const_trait_impl", issue = "143874")]
pub const trait Fn<Args: Tuple>: [const] FnMut<Args> {
    /// Performs the call operation.
    #[unstable(feature = "fn_traits", issue = "29625")]
    extern "rust-call" fn call(&self, args: Args) -> Self::Output;
}

#[unstable(feature="core_intrinsics",issue="none")]
pub mod intrinsics{use super::{Tuple,FnOnce};
#[rustc_const_unstable(feature = "const_eval_select", issue = "124625")]
#[rustc_intrinsic]
pub const fn const_eval_select<ARG: Tuple, F, G, RET>(
    _arg: ARG,
    _called_in_const: F,
    _called_at_rt: G,
) -> RET
where
    G: FnOnce<ARG, Output = RET>,
    F: const FnOnce<ARG, Output = RET>;

}
const fn ct()->u8{1}fn rt()->u8{7}#[stable(feature="source_headers",since="1.0.0")]#[unsafe(no_mangle)]pub extern "C" fn answer()->u8{intrinsics::const_eval_select((),ct,rt)}
