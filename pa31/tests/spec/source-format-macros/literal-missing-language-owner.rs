#![feature(no_core,lang_items,rustc_attrs,allow_internal_unsafe,allow_internal_unstable)]
#![no_core]
#[lang="pointee_sized"] pub trait PointeeSized {}
#[lang="meta_sized"] pub trait MetaSized:PointeeSized {}
#[lang="sized"] pub trait Sized:MetaSized {}
#[lang="clone"] pub trait Clone:Sized { #[lang="clone_fn"] fn clone(&self)->Self; }
#[lang="copy"] pub trait Copy:Clone {}
#[lang="legacy_receiver"] pub trait LegacyReceiver:PointeeSized {}
impl<T:PointeeSized> LegacyReceiver for &T {}
impl<T:PointeeSized> LegacyReceiver for &mut T {}
impl Clone for u32 { fn clone(&self)->Self { *self } }
impl Copy for u32 {}
#[rustc_builtin_macro]
macro_rules! format_args { ($($tt:tt)*) => {{ /* compiler built-in */ }}; }
pub struct TextArguments<'a> { pub text:&'a str }
impl<'a> TextArguments<'a> {
    pub const fn from_str(text:&'static str)->Self { Self { text } }
}
#[unsafe(no_mangle)] pub extern "C" fn answer()->*const u8 { format_args!("hello").text as *const str as *const u8 }
