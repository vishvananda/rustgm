#![feature(no_core, lang_items, rustc_attrs, staged_api, auto_traits)]
#![no_core]
#![unstable(feature = "disc_fixture", issue = "none")]
#![allow(internal_features)]

#[lang = "pointee_sized"]
pub trait PointeeSized {}
#[lang = "meta_sized"]
pub trait MetaSized: PointeeSized {}
#[lang = "sized"]
pub trait Sized: MetaSized {}
#[lang = "legacy_receiver"]
pub trait LegacyReceiver: PointeeSized {}
impl<T: PointeeSized> LegacyReceiver for &T {}
impl<T: PointeeSized> LegacyReceiver for &mut T {}

#[lang = "clone"]
pub trait Clone: Sized {
    #[lang = "clone_fn"]
    fn clone(&self) -> Self;
}
#[lang = "copy"]
pub trait Copy: Clone {}
impl Clone for u8 { fn clone(&self) -> Self { *self } }
impl Copy for u8 {}
impl Clone for u16 { fn clone(&self) -> Self { *self } }
impl Copy for u16 {}
impl Clone for isize { fn clone(&self) -> Self { *self } }
impl Copy for isize {}

pub trait Debug {}
pub trait Eq {}
pub trait PartialEq {}
pub trait Hash {}
impl<T> Debug for T {}
impl<T> Eq for T {}
impl<T> PartialEq for T {}
impl<T> Hash for T {}
#[rustc_diagnostic_item = "Send"]
pub unsafe auto trait Send {}
#[lang = "sync"]
pub unsafe auto trait Sync {}
#[lang = "freeze"]
pub unsafe auto trait Freeze {}
#[lang = "unpin"]
pub auto trait Unpin {}

// DiscriminantKind and its result bounds are from pinned core/src/marker.rs.
#[unstable(feature = "discriminant_kind", issue = "none", reason = "this trait is unlikely to ever be stabilized, use `mem::discriminant` instead")]
#[lang = "discriminant_kind"]
#[rustc_deny_explicit_impl]
#[rustc_dyn_incompatible_trait]
pub trait DiscriminantKind {
    #[lang = "discriminant_type"]
    type Discriminant: Clone + Copy + Debug + Eq + PartialEq + Hash + Send + Sync + Unpin;
}

#[repr(u16)]
pub enum ExternalTag { First = 31, Second = 513 }
pub struct ExternalRecord(pub u8);
