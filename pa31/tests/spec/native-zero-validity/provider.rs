#![feature(no_core, lang_items, intrinsics, rustc_attrs)]
#![no_core]
#![cfg_attr(rustc_host, no_main)]

#[lang = "pointee_sized"]
pub trait PointeeSized {}
#[lang = "meta_sized"]
pub trait MetaSized: PointeeSized {}
#[lang = "sized"]
pub trait Sized: MetaSized {}
#[lang = "legacy_receiver"]
pub trait LegacyReceiver: PointeeSized {}
impl<T: PointeeSized> LegacyReceiver for &T {}
#[lang = "clone"]
pub trait Clone: Sized {
    #[lang = "clone_fn"]
    fn clone(&self) -> Self;
}
#[lang = "copy"]
pub trait Copy: Clone {}
impl Clone for i32 {
    fn clone(&self) -> Self { *self }
}
impl Copy for i32 {}


pub mod prelude {
    pub mod rust_2024 {}
}

#[rustc_intrinsic]
const fn assert_zero_valid<T>();

#[link(name = "c")]
unsafe extern "C" {
    fn _Exit(status: i32) -> !;
}

#[lang = "panic_nounwind"]
pub fn report(_: &'static str) -> ! {
    unsafe { _Exit(73) }
}

struct Header {
    count: u32,
    enabled: bool,
    raw: *const u8,
    storage: [u16; 3],
}

#[repr(i8)]
enum SpansZero {
    Negative = 254u8 as i8,
    Positive = 2,
}

#[repr(u8)]
enum ExcludesZero {
    A = 1,
    B = 3,
}

pub fn probe(argc: i32) -> i32 {
    match argc {
        1 => {
            assert_zero_valid::<Header>();
            assert_zero_valid::<[*const u8; 2]>();
            assert_zero_valid::<[&'static u8; 1]>();
            assert_zero_valid::<SpansZero>();
        }
        2 => assert_zero_valid::<&'static u8>(),
        _ => assert_zero_valid::<ExcludesZero>(),
    }
    0
}

#[cfg(rustc_host)]
#[unsafe(no_mangle)]
pub extern "C" fn main(argc: i32, _argv: *const *const u8) -> i32 {
    probe(argc)
}
