#![feature(no_core,lang_items,rustc_attrs,staged_api,auto_traits,const_trait_impl,intrinsics,inherent_associated_types,fundamental,field_projections)]
#![no_core]
#![unstable(feature="reborrow_fixture",issue="none")]
#![allow(internal_features,dead_code)]
#[lang="pointee_sized"]pub trait PointeeSized{}
#[lang="meta_sized"]pub trait MetaSized:PointeeSized{}
#[lang="sized"]pub trait Sized:MetaSized{}
#[lang="legacy_receiver"]pub trait LegacyReceiver:PointeeSized{}
impl<T:PointeeSized>LegacyReceiver for &T{}
impl<T:PointeeSized>LegacyReceiver for &mut T{}
#[lang="freeze"]pub unsafe auto trait Freeze{}
#[rustc_const_unstable(feature="const_destruct",issue="133214")]#[lang="destruct"]#[rustc_deny_explicit_impl]pub const trait Destruct:PointeeSized{}
#[lang="drop_glue"]pub const unsafe fn drop_glue<T:PointeeSized>(_:&mut T)where T:[const]Destruct{}
#[lang="clone"]pub trait Clone:Sized{#[lang="clone_fn"]fn clone(&self)->Self;}
#[lang="copy"]pub trait Copy:Clone{}
impl Clone for u8{fn clone(&self)->Self{*self}}impl Copy for u8{}
impl<T:PointeeSized>Clone for &T{fn clone(&self)->Self{*self}}impl<T:PointeeSized>Copy for &T{}
#[rustc_diagnostic_item = "Send"]
pub unsafe auto trait Send{}
#[lang="sync"]pub unsafe auto trait Sync{}
pub mod marker { #[lang="phantom_data"]pub struct PhantomData<T:super::PointeeSized>; }
impl<T:PointeeSized>Clone for marker::PhantomData<T>{fn clone(&self)->Self{marker::PhantomData}}
impl<T:PointeeSized>Copy for marker::PhantomData<T>{}
impl Clone for u32{fn clone(&self)->Self{*self}}impl Copy for u32{}
impl Clone for usize{fn clone(&self)->Self{*self}}impl Copy for usize{}
pub mod field {
use crate::{Sized,Send,Sync,Copy,Clone,marker::PhantomData};
/// Field Representing Type
#[unstable(feature = "field_representing_type_raw", issue = "none")]
#[lang = "field_representing_type"]
#[fundamental]
pub struct FieldRepresentingType<T: ?Sized, const VARIANT: u32, const FIELD: u32> {
    // We want this type to be invariant over `T`, because otherwise `field_of!(Struct<'short>,
    // field)` is a subtype of `field_of!(Struct<'long>, field)`. This subtype relationship does not
    // have an immediately obvious meaning and we want to prevent people from relying on it.
    _phantom: PhantomData<fn(T) -> T>,
}

/// Type representing a field of a `struct`, `union`, `enum` variant or tuple.
///
/// # Safety
///
/// Given a valid value of type `Self::Base`, there exists a valid value of type `Self::Type` at
/// byte offset `OFFSET`
#[lang = "field"]
#[unstable(feature = "field_projections", issue = "145383")]
#[rustc_deny_explicit_impl]
#[rustc_dyn_incompatible_trait]
// NOTE: the compiler provides the impl of `Field` for `FieldRepresentingType` when it can guarantee
// the safety requirements of this trait. It also has to manually add the correct trait bounds on
// associated types (and the `Self` type). Thus any changes to the bounds here must be reflected in
// the old and new trait solver:
// - `fn assemble_candidates_for_field_trait` in
//   `compiler/rustc_trait_selection/src/traits/select/candidate_assembly.rs`, and
// - `fn consider_builtin_field_candidate` in
//   `compiler/rustc_next_trait_solver/src/solve/trait_goals.rs`.
pub unsafe trait Field: Send + Sync + Copy {
    /// The type of the base where this field exists in.
    #[lang = "field_base"]
    type Base;

    /// The type of the field.
    #[lang = "field_type"]
    type Type;

    /// The offset of the field in bytes.
    #[lang = "field_offset"]
    const OFFSET: usize = crate::intrinsics::field_offset::<Self>();
}

impl<T: ?Sized, const VARIANT: u32, const FIELD: u32> Copy
    for FieldRepresentingType<T, VARIANT, FIELD>
{
}

impl<T: ?Sized, const VARIANT: u32, const FIELD: u32> Clone
    for FieldRepresentingType<T, VARIANT, FIELD>
{
    fn clone(&self) -> Self {
        *self
    }
}
}
pub mod intrinsics {
#[rustc_intrinsic]
#[unstable(feature = "field_projections", issue = "145383")]
#[rustc_const_unstable(feature = "field_projections", issue = "145383")]
pub const fn field_offset<F: crate::field::Field>() -> usize;
}
pub struct Tail{first:u8,tail:[u8]}fn need<F:field::Field>(){}pub fn check(){need::<field::FieldRepresentingType<Tail,0,1>>();}
