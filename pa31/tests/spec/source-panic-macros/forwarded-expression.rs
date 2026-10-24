#![feature(no_core, lang_items, rustc_attrs)]
#![no_core]
#[lang = "pointee_sized"] pub trait PointeeSized {}
#[lang = "meta_sized"] pub trait MetaSized: PointeeSized {}
#[lang = "sized"] pub trait Sized: MetaSized {}
macro_rules! seed { ($value:expr) => { $value }; }
const SEED: u32 = seed!(3u32);
pub mod panic {
    macro_rules! chosen { ($value:expr) => { 7u32 }; }
    pub(crate) use chosen as panic_2021;
}
#[rustc_builtin_macro(core_panic)]
macro_rules! route { ($($input:tt)*) => { 9u32 }; }
macro_rules! wrapper { ($value:expr) => { route!($value) }; }
#[unsafe(no_mangle)] pub extern "C" fn answer() -> u32 { wrapper!(7u32) }
