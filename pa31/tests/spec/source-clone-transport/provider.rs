// Authored reduction of the pinned Rust 1.98.0 core/src/clone.rs declarations.
#![feature(no_core, lang_items, rustc_attrs, staged_api, marker_trait_attr,
           const_trait_impl, const_destruct, auto_traits)]
#![no_core]
#![unstable(feature="clone_fixture", issue="none")]
#![allow(internal_features, incomplete_features)]

#[lang="pointee_sized"]
pub trait PointeeSized {}
#[lang="meta_sized"]
pub trait MetaSized: PointeeSized {}
#[lang="sized"]
pub trait Sized: MetaSized {}
#[lang="legacy_receiver"]
pub trait LegacyReceiver: PointeeSized {}
impl<T: PointeeSized> LegacyReceiver for &T {}
impl<T: PointeeSized> LegacyReceiver for &mut T {}
#[lang="freeze"]
pub unsafe auto trait Freeze {}

#[rustc_const_unstable(feature="const_destruct", issue="133214")]
#[lang="destruct"]
#[rustc_deny_explicit_impl]
#[rustc_dyn_incompatible_trait]
pub const trait Destruct: PointeeSized {}
#[lang="drop_glue"]
pub const unsafe fn drop_glue<T: PointeeSized>(_: &mut T)
where
    T: [const] Destruct,
{}

#[stable(feature="rust1", since="1.0.0")]
#[lang="clone"]
#[rustc_diagnostic_item="Clone"]
#[rustc_trivial_field_reads]
#[rustc_const_unstable(feature="const_clone", issue="142757")]
pub const trait Clone: Sized {
    #[stable(feature="rust1", since="1.0.0")]
    #[must_use="cloning is often expensive and is not expected to have side effects"]
    #[lang="clone_fn"]
    fn clone(&self) -> Self;
}

#[lang="copy"]
pub trait Copy: Clone {}
impl Clone for u8 {
    fn clone(&self) -> Self { *self }
}
impl Copy for u8 {}

#[unstable(feature="trivial_clone", reason="this isn't part of any API guarantee", issue="none")]
#[rustc_const_unstable(feature="const_clone", issue="142757")]
#[lang="trivial_clone"]
#[rustc_unsafe_specialization_marker]
#[marker]
pub const unsafe trait TrivialClone: [const] Clone {}
