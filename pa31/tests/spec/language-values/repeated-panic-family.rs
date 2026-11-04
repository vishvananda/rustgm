#![feature(no_core,lang_items,rustc_attrs)]
#![no_core]
#![allow(internal_features,unused_macros,dead_code)]
#[lang="pointee_sized"]pub trait PointeeSized{}
#[lang="meta_sized"]pub trait MetaSized:PointeeSized{}
#[lang="sized"]pub trait Sized:MetaSized{}
#[macro_export]#[rustc_builtin_macro]macro_rules! concat {()=>{}}
#[macro_export]#[rustc_builtin_macro]macro_rules! stringify {()=>{}}
macro_rules! declare {($($name:ident),*)=>{$(#[lang=stringify!($name)]pub const fn $name()->!{loop{}})*};}
declare!(panic_const_add_overflow,panic_const_sub_overflow,panic_const_mul_overflow,panic_const_div_overflow,panic_const_rem_overflow,panic_const_neg_overflow,panic_const_shr_overflow,panic_const_shl_overflow,panic_const_div_by_zero,panic_const_rem_by_zero,panic_const_coroutine_resumed,panic_const_async_fn_resumed,panic_const_async_gen_fn_resumed,panic_const_gen_fn_none,panic_const_coroutine_resumed_panic,panic_const_async_fn_resumed_panic,panic_const_async_gen_fn_resumed_panic,panic_const_gen_fn_none_panic,panic_const_coroutine_resumed_drop,panic_const_async_fn_resumed_drop,panic_const_async_gen_fn_resumed_drop,panic_const_gen_fn_none_drop);
