#![feature(no_core,lang_items,rustc_attrs,auto_traits,min_adt_const_params)]
#![no_core]
#![allow(internal_features,dead_code)]
#[lang="pointee_sized"] pub trait PointeeSized {}
#[lang="meta_sized"] pub trait MetaSized:PointeeSized {}
#[lang="sized"] pub trait Sized:MetaSized {}
#[lang="legacy_receiver"] pub trait LegacyReceiver:PointeeSized {}
impl<T:PointeeSized> LegacyReceiver for &T {}
impl<T:PointeeSized> LegacyReceiver for &mut T {}
#[lang="freeze"] pub unsafe auto trait Freeze {}
#[lang="clone"] pub trait Clone:Sized { #[lang="clone_fn"] fn clone(&self)->Self; }
#[lang="copy"] pub trait Copy:Clone {}
impl Clone for bool { fn clone(&self)->Self {*self} }
impl Copy for bool {}
impl Clone for u8 { fn clone(&self)->Self {*self} }
impl Copy for u8 {}
#[lang="structural_peq"] pub trait Structural {}
#[rustc_diagnostic_item="Eq"] pub trait Equal {}
#[lang="const_param_ty"] pub trait Key:Structural+Equal {}
impl Structural for bool {}
impl Equal for bool {}
impl Key for bool {}
impl Structural for usize{} impl Equal for usize{} impl Key for usize{}
pub trait Owner{#[lang="fn_ptr_addr"]fn addr(&self){}}
