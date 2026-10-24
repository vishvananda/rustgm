#![feature(no_core, lang_items, rustc_attrs, decl_macro)]
#![no_core]
#[lang = "pointee_sized"] pub trait PointeeSized {}
#[lang = "meta_sized"] pub trait MetaSized: PointeeSized {}
#[lang = "sized"] pub trait Sized: MetaSized {}
#[rustc_builtin_macro]
pub macro cfg_select($($tt:tt)*) { /* compiler built-in */ }
trait Value { fn value() -> u32; }
struct Marker;
impl Value for Marker { cfg_select! { all() => { fn value() -> u32 { 7u32 } } _ => { fn value() -> u32 { missing_name } } } }
#[unsafe(no_mangle)] pub extern "C" fn answer() -> u32 { <Marker as Value>::value() }
