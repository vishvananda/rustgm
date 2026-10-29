#![feature(const_trait_impl,const_destruct,auto_traits)]
#![feature(staged_api)]
#![stable(feature="availability_provider",since="1.0.0")]
#![feature(no_core,lang_items,rustc_attrs,fundamental)]
#![no_core]
#![allow(internal_features,dead_code,unused_variables)]
#[stable(feature="availability_roles",since="1.0.0")]
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
#[stable(feature="availability_roles",since="1.0.0")]
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
#[stable(feature="availability_roles",since="1.0.0")]
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
#[stable(feature="availability_roles",since="1.0.0")]
#[lang = "legacy_receiver"]
pub trait LegacyReceiver: PointeeSized {}
#[stable(feature="availability_roles",since="1.0.0")]
impl<T: PointeeSized> LegacyReceiver for &T {}
#[stable(feature="availability_roles",since="1.0.0")]
impl<T: PointeeSized> LegacyReceiver for &mut T {}
#[stable(feature="availability_roles",since="1.0.0")]
#[lang = "clone"]
pub trait Clone: Sized { #[stable(feature="availability_roles",since="1.0.0")] #[lang = "clone_fn"] fn clone(&self) -> Self; }
#[stable(feature="availability_roles",since="1.0.0")]
#[lang = "copy"]
pub trait Copy: Clone {}
#[unstable(feature="compat_mod",issue="none")]
pub mod gated {
 #[stable(feature="compat_leaf",since="1.0.0")]#[rustc_allowed_through_unstable_modules="use the stable alias"]pub fn marked()->u8{7}
 #[stable(feature="compat_leaf",since="1.0.0")]pub fn plain()->u8{9}
 #[unstable(feature="compat_leaf_gate",issue="none")]pub fn unstable_leaf()->u8{11}
 #[stable(feature="compat_leaf",since="1.0.0")]#[rustc_allowed_through_unstable_modules="use the stable alias"]#[rustc_const_unstable(feature="compat_const",issue="none")]pub const fn const_marked()->u8{7}
 #[stable(feature="compat_leaf",since="1.0.0")]#[rustc_allowed_through_unstable_modules="use the stable alias"]pub unsafe fn unsafe_marked()->u8{7}
 #[unstable(feature="compat_inner",issue="none")]pub mod inner{#[stable(feature="compat_leaf",since="1.0.0")]#[rustc_allowed_through_unstable_modules="use the stable alias"]pub fn marked()->u8{7}}
}
#[stable(feature="compat_leaf",since="1.0.0")]pub use gated::marked as stable_alias;
#[stable(feature="compat_leaf",since="1.0.0")]pub fn local()->u8{gated::marked()}

#[lang = "freeze"]
#[unstable(feature = "freeze", issue = "121675")]
pub unsafe auto trait Freeze {}

#[unstable(feature = "const_destruct", issue = "133214")]
#[rustc_const_unstable(feature = "const_destruct", issue = "133214")]
#[lang = "destruct"]
#[diagnostic::on_unimplemented(message = "can't drop `{Self}`")]
#[rustc_deny_explicit_impl]
#[rustc_dyn_incompatible_trait]
pub const trait Destruct: PointeeSized {}

// Authored closed-body protocol, not replacements for original library algorithms.
#[stable(feature="callable_effects",since="1.0.0")]
#[rustc_const_stable(feature="callable_effects",since="1.0.0")]
pub const unsafe fn const_unsafe(value:u8)->u8{value}
#[stable(feature="callable_effects",since="1.0.0")]
#[rustc_const_stable(feature="callable_effects",since="1.0.0")]
pub const fn const_identity<T>(value:T)->T{value}
#[stable(feature="callable_effects",since="1.0.0")]
#[rustc_const_stable(feature="callable_effects",since="1.0.0")]
pub const fn const_forward(value:u8)->u8{const_identity(value)}
