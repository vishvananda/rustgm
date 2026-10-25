#![feature(no_core,lang_items,rustc_attrs,decl_macro)]
#![no_core]
#[lang="pointee_sized"]pub trait PointeeSized{}
#[lang="meta_sized"]pub trait MetaSized:PointeeSized{}
#[lang="sized"]pub trait Sized:MetaSized{}
#[rustc_builtin_macro(autodiff_forward)]
pub macro actual_0($item:item){ /* compiler built-in */ }
#[rustc_builtin_macro(autodiff_forward)]
pub macro actual_1($item:item){ /* compiler built-in */ }
#[rustc_builtin_macro(autodiff_forward)]
pub macro actual_2($item:item){ /* compiler built-in */ }
#[rustc_builtin_macro(autodiff_forward)]
pub macro actual_3($item:item){ /* compiler built-in */ }
#[rustc_builtin_macro(autodiff_forward)]
pub macro actual_4($item:item){ /* compiler built-in */ }
#[rustc_builtin_macro(autodiff_forward)]
pub macro actual_5($item:item){ /* compiler built-in */ }
#[rustc_builtin_macro(autodiff_forward)]
pub macro actual_6($item:item){ /* compiler built-in */ }
#[rustc_builtin_macro(autodiff_forward)]
pub macro actual_7($item:item){ /* compiler built-in */ }
#[unsafe(no_mangle)]pub extern "C" fn answer()->u32{7u32}
