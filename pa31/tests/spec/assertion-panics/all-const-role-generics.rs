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
#[lang="panic_const_add_overflow"]pub fn marked_187<T>(x:T)->T{x}
#[lang="panic_const_sub_overflow"]pub fn marked_188<T>(x:T)->T{x}
#[lang="panic_const_mul_overflow"]pub fn marked_189<T>(x:T)->T{x}
#[lang="panic_const_div_overflow"]pub fn marked_190<T>(x:T)->T{x}
#[lang="panic_const_rem_overflow"]pub fn marked_191<T>(x:T)->T{x}
#[lang="panic_const_neg_overflow"]pub fn marked_192<T>(x:T)->T{x}
#[lang="panic_const_shr_overflow"]pub fn marked_193<T>(x:T)->T{x}
#[lang="panic_const_shl_overflow"]pub fn marked_194<T>(x:T)->T{x}
#[lang="panic_const_div_by_zero"]pub fn marked_195<T>(x:T)->T{x}
#[lang="panic_const_rem_by_zero"]pub fn marked_196<T>(x:T)->T{x}
#[lang="panic_const_coroutine_resumed"]pub fn marked_197<T>(x:T)->T{x}
#[lang="panic_const_async_fn_resumed"]pub fn marked_198<T>(x:T)->T{x}
#[lang="panic_const_async_gen_fn_resumed"]pub fn marked_199<T>(x:T)->T{x}
#[lang="panic_const_gen_fn_none"]pub fn marked_200<T>(x:T)->T{x}
#[lang="panic_const_coroutine_resumed_panic"]pub fn marked_201<T>(x:T)->T{x}
#[lang="panic_const_async_fn_resumed_panic"]pub fn marked_202<T>(x:T)->T{x}
#[lang="panic_const_async_gen_fn_resumed_panic"]pub fn marked_203<T>(x:T)->T{x}
#[lang="panic_const_gen_fn_none_panic"]pub fn marked_204<T>(x:T)->T{x}
#[lang="panic_const_coroutine_resumed_drop"]pub fn marked_205<T>(x:T)->T{x}
#[lang="panic_const_async_fn_resumed_drop"]pub fn marked_206<T>(x:T)->T{x}
#[lang="panic_const_async_gen_fn_resumed_drop"]pub fn marked_207<T>(x:T)->T{x}
#[lang="panic_const_gen_fn_none_drop"]pub fn marked_208<T>(x:T)->T{x}
