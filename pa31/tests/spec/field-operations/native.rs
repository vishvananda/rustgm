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
#[repr(C)]pub struct Record{byte:u8,word:u32}
pub struct Generic<T>{head:u8,value:T}
pub struct Borrowed<'a>{head:u8,value:&'a u8}
pub mod pointer_intrinsics{
#[must_use = "returns a new pointer rather than modifying its argument"]
#[rustc_intrinsic_const_stable_indirect]
#[rustc_nounwind]
#[rustc_intrinsic]
pub const unsafe fn arith_offset<T>(dst: *const T, offset: isize) -> *const T;
}
use field::{Field,FieldRepresentingType as FRT};
fn project<'a,F:Field>(base:&'a F::Base)->&'a F::Type{
    let offset=const{F::OFFSET};
    unsafe{&*(pointer_intrinsics::arith_offset(base as*const F::Base as*const u8,offset as isize)as*const F::Type)}
}
pub struct RustRecord{byte:u8,word:u32}
fn generic<T:Copy>(value:T)->T{let r=Generic{head:0,value};*project::<FRT<Generic<T>,0,1>>(&r)}
fn borrowed<'a>(base:&Borrowed<'a>)->&'a u8{*project::<FRT<Borrowed<'a>,0,1>>(base)}
unsafe extern "C"{fn field_value()->u32;}
#[unsafe(no_mangle)]pub extern "C" fn field_offset_c()->usize{const{<FRT<Record,0,1>as Field>::OFFSET}}
#[unsafe(no_mangle)]pub extern "C" fn field_c(byte:u8,word:u32)->u32{let r=Record{byte,word};*project::<FRT<Record,0,1>>(&r)}
#[unsafe(no_mangle)]pub extern "C" fn field_rust(byte:u8,word:u32)->u32{let r=RustRecord{byte,word};*project::<FRT<RustRecord,0,1>>(&r)}
#[unsafe(no_mangle)]pub extern "C" fn field_tuple(byte:u8,word:u32)->u32{let r=(byte,word);*project::<FRT<(u8,u32),0,1>>(&r)}
#[unsafe(no_mangle)]pub extern "C" fn field_generic(word:u32)->u32{generic(word)}
#[unsafe(no_mangle)]pub unsafe extern "C" fn field_borrowed(ptr:*const u8)->u8{let value=unsafe{&*ptr};let r=Borrowed{head:0,value};*borrowed(&r)}
#[unsafe(no_mangle)]pub extern "C" fn field_effect()->u32{let r=Generic{head:0,value:unsafe{field_value()}};*project::<FRT<Generic<u32>,0,1>>(&r)}
