#![feature(no_core,lang_items,intrinsics,rustc_attrs,cfg_ub_checks)]
#![no_core]
#![rustc_preserve_ub_checks]
#[rustc_builtin_macro]macro_rules!cfg{($($t:tt)*)=>{}}
#[lang="pointee_sized"]pub trait PointeeSized{}
#[lang="meta_sized"]pub trait MetaSized:PointeeSized{}
#[lang="sized"]pub trait Sized:MetaSized{}
pub const fn ub_checks()->bool{false}
const CT:bool=ub_checks();
type Probe=[u8;ub_checks() as usize];
fn generic<T:Sized>()->bool{ub_checks()}
#[unsafe(no_mangle)]pub extern "C" fn constant()->u8{let _:Probe=[0u8;0];CT as u8}
#[unsafe(no_mangle)]pub extern "C" fn runtime()->u8{ub_checks() as u8}
#[unsafe(no_mangle)]pub extern "C" fn configured()->u8{cfg!(ub_checks) as u8}
#[unsafe(no_mangle)]pub extern "C" fn generic_runtime()->u8{generic::<u8>() as u8}
