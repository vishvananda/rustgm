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
#[stable(feature="identity_api",since="1.0.0")]
pub fn identity<T>(x: T) -> T { x }

#[unstable(feature="experimental_identity",issue="none")]
pub fn experimental<T>(x:T)->T{x}
#[stable(feature="bridge_api",since="1.0.0")]
pub mod bridge {
#[unstable(feature="bridge_identity",issue="none")]
pub use crate::identity as forwarded;
}

fn private_identity<T>(x:T)->T{x}

#[stable(feature="routes",since="1.0.0")]pub mod routes{
#[stable(feature="stable_alias",since="1.0.0")]pub use crate::experimental as stable_alias;
#[unstable(feature="alias_unstable",issue="none")]pub use crate::experimental as unstable_alias;
}

#[unstable(feature="module_identity",issue="none")]pub mod gated{
#[stable(feature="gated_forward",since="1.0.0")]pub use crate::identity as forwarded;
}

#[unstable(feature="predicate_probe",issue="none")]
#[unstable_feature_bound(predicate_probe)]
pub trait PredicateGate {}
#[unstable(feature="predicate_probe",issue="none")]
#[unstable_feature_bound(predicate_probe)]
impl PredicateGate for u8 {}
#[unstable(feature="predicate_api",issue="none")]
#[unstable_feature_bound(predicate_probe)]
pub fn gated<T>(x:T)->T{x}
#[unstable(feature="predicate_api",issue="none")]
#[unstable_feature_bound(scope_only)]
pub fn scoped<T>(x:T)->T{x}

#[unstable(feature="dispatch_probe",issue="none")]
#[unstable_feature_bound(dispatch_probe)]
pub trait PrimitiveDispatch { fn value()->u8; }
#[unstable(feature="dispatch_probe",issue="none")]
#[unstable_feature_bound(dispatch_probe)]
impl PrimitiveDispatch for u8 { fn value()->u8{7u8} }
#[unstable(feature="dispatch_api",issue="none")]
#[unstable_feature_bound(dispatch_probe)]
pub fn dispatch()->u8{<u8 as PrimitiveDispatch>::value()}
