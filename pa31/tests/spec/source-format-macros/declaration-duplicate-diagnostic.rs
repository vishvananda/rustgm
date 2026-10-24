#![feature(no_core,lang_items,rustc_attrs,allow_internal_unsafe,allow_internal_unstable)]
#![no_core]
#[lang="pointee_sized"] pub trait PointeeSized {}
#[lang="meta_sized"] pub trait MetaSized:PointeeSized {}
#[lang="sized"] pub trait Sized:MetaSized {}
#[rustc_diagnostic_item="format_args_macro"]
#[rustc_builtin_macro(format_args)]
macro_rules! first { ($($tt:tt)*) => {{ /* compiler built-in */ }}; }
#[rustc_diagnostic_item="format_args_macro"]
#[rustc_builtin_macro(format_args)]
macro_rules! second { ($($tt:tt)*) => {{ /* compiler built-in */ }}; }
#[unsafe(no_mangle)] pub extern "C" fn answer()->u32 { 7u32 }
