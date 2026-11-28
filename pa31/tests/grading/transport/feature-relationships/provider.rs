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
#[unstable(feature = "controlling_api", issue = "none")]
pub fn controlling() -> u8 { 1 }

#[unstable(feature = "dependent_api", issue = "none", implied_by = "controlling_api")]
pub fn dependent() -> u8 { 2 }

#[unstable(feature = "current_api", issue = "none", old_name = "former_api")]
pub fn renamed() -> u8 { 3 }

#[unstable(feature = "chained_api", issue = "none", implied_by = "dependent_api")]
pub fn chained() -> u8 { 4 }

#[unstable(feature = "described_api", issue = "none", old_name = "former spelling")]
pub fn described() -> u8 { 5 }
