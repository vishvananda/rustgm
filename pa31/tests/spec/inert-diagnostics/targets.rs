#![rustc_diagnostic_item="inert_crate"]
#![feature(const_trait_impl)]
#![feature(no_core,lang_items,rustc_attrs,fundamental)]
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
pub trait LegacyReceiver: PointeeSized {}
impl<T: PointeeSized> LegacyReceiver for &T {}
impl<T: PointeeSized> LegacyReceiver for &mut T {}
#[lang = "clone"]
pub trait Clone: Sized { #[lang = "clone_fn"] fn clone(&self) -> Self; }
#[lang = "copy"]
pub trait Copy: Clone {}

#[lang="destruct"]#[rustc_deny_explicit_impl]#[rustc_dyn_incompatible_trait]
pub const trait Destruct:PointeeSized{}
impl Clone for isize{fn clone(&self)->Self{*self}}impl Copy for isize{}
#[rustc_diagnostic_item="inert_fn"]fn f(){}
#[rustc_diagnostic_item="inert_struct"]struct S;
#[rustc_diagnostic_item="inert_enum"]enum E{V}
#[rustc_diagnostic_item="inert_alias"]type A=S;
#[rustc_diagnostic_item="inert_const"]const C:u8=0;
#[rustc_diagnostic_item="inert_mod"]mod inner{}
#[rustc_diagnostic_item="inert_macro"]macro_rules! m{()=>{}}
#[rustc_diagnostic_item="inert_trait"]trait T{
#[rustc_diagnostic_item="inert_assoc_ty"]type A;
#[rustc_diagnostic_item="inert_assoc_const"]const N:u8;
#[rustc_diagnostic_item="inert_required"]fn required();
#[rustc_diagnostic_item="inert_default"]fn defaulted(){}
}
#[rustc_diagnostic_item="inert_impl"]impl S{#[rustc_diagnostic_item="inert_inherent"]fn value(){}}
impl T for S{type A=();const N:u8=0;#[rustc_diagnostic_item="inert_override"]fn required(){}}
