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
#[lang="panic_bounds_check"]#[track_caller]pub fn hook_138(index:usize,len:usize)->!{loop{}}
#[lang="panic_misaligned_pointer_dereference"]#[track_caller]#[rustc_nounwind]pub fn hook_139(required:usize,found:usize)->!{loop{}}
#[lang="panic_cannot_unwind"]#[rustc_nounwind]pub fn hook_140()->!{loop{}}
#[lang="panic_in_cleanup"]#[rustc_nounwind]pub fn hook_141()->!{loop{}}
#[lang="panic_null_pointer_dereference"]#[track_caller]#[rustc_nounwind]pub fn hook_142()->!{loop{}}
#[lang="panic_invalid_enum_construction"]#[track_caller]#[rustc_nounwind]pub fn hook_143(source:u128)->!{loop{}}
#[lang="panic_const_add_overflow"]#[track_caller]pub const fn hook_187()->!{loop{}}
#[lang="panic_const_sub_overflow"]#[track_caller]pub const fn hook_188()->!{loop{}}
#[lang="panic_const_mul_overflow"]#[track_caller]pub const fn hook_189()->!{loop{}}
#[lang="panic_const_div_overflow"]#[track_caller]pub const fn hook_190()->!{loop{}}
#[lang="panic_const_rem_overflow"]#[track_caller]pub const fn hook_191()->!{loop{}}
#[lang="panic_const_neg_overflow"]#[track_caller]pub const fn hook_192()->!{loop{}}
#[lang="panic_const_shr_overflow"]#[track_caller]pub const fn hook_193()->!{loop{}}
#[lang="panic_const_shl_overflow"]#[track_caller]pub const fn hook_194()->!{loop{}}
#[lang="panic_const_div_by_zero"]#[track_caller]pub const fn hook_195()->!{loop{}}
#[lang="panic_const_rem_by_zero"]#[track_caller]pub const fn hook_196()->!{loop{}}
#[lang="panic_const_coroutine_resumed"]#[track_caller]pub const fn hook_197()->!{loop{}}
#[lang="panic_const_async_fn_resumed"]#[track_caller]pub const fn hook_198()->!{loop{}}
#[lang="panic_const_async_gen_fn_resumed"]#[track_caller]pub const fn hook_199()->!{loop{}}
#[lang="panic_const_gen_fn_none"]#[track_caller]pub const fn hook_200()->!{loop{}}
#[lang="panic_const_coroutine_resumed_panic"]#[track_caller]pub const fn hook_201()->!{loop{}}
#[lang="panic_const_async_fn_resumed_panic"]#[track_caller]pub const fn hook_202()->!{loop{}}
#[lang="panic_const_async_gen_fn_resumed_panic"]#[track_caller]pub const fn hook_203()->!{loop{}}
#[lang="panic_const_gen_fn_none_panic"]#[track_caller]pub const fn hook_204()->!{loop{}}
#[lang="panic_const_coroutine_resumed_drop"]#[track_caller]pub const fn hook_205()->!{loop{}}
#[lang="panic_const_async_fn_resumed_drop"]#[track_caller]pub const fn hook_206()->!{loop{}}
#[lang="panic_const_async_gen_fn_resumed_drop"]#[track_caller]pub const fn hook_207()->!{loop{}}
#[lang="panic_const_gen_fn_none_drop"]#[track_caller]pub const fn hook_208()->!{loop{}}
