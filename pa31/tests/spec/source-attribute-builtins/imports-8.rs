#![feature(no_core,lang_items,rustc_attrs,decl_macro)]
#![no_core]
#[lang="pointee_sized"]pub trait PointeeSized{}
#[lang="meta_sized"]pub trait MetaSized:PointeeSized{}
#[lang="sized"]pub trait Sized:MetaSized{}
mod holder{#[rustc_builtin_macro(autodiff_forward)]
pub macro actual($item:item){ /* compiler built-in */ }
}
use holder::actual as alias_0;
use holder::actual as alias_1;
use holder::actual as alias_2;
use holder::actual as alias_3;
use holder::actual as alias_4;
use holder::actual as alias_5;
use holder::actual as alias_6;
use holder::actual as alias_7;
#[unsafe(no_mangle)]pub extern "C" fn answer()->u32{7u32}
