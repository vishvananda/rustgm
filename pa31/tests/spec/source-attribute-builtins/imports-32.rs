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
use holder::actual as alias_8;
use holder::actual as alias_9;
use holder::actual as alias_10;
use holder::actual as alias_11;
use holder::actual as alias_12;
use holder::actual as alias_13;
use holder::actual as alias_14;
use holder::actual as alias_15;
use holder::actual as alias_16;
use holder::actual as alias_17;
use holder::actual as alias_18;
use holder::actual as alias_19;
use holder::actual as alias_20;
use holder::actual as alias_21;
use holder::actual as alias_22;
use holder::actual as alias_23;
use holder::actual as alias_24;
use holder::actual as alias_25;
use holder::actual as alias_26;
use holder::actual as alias_27;
use holder::actual as alias_28;
use holder::actual as alias_29;
use holder::actual as alias_30;
use holder::actual as alias_31;
#[unsafe(no_mangle)]pub extern "C" fn answer()->u32{7u32}
