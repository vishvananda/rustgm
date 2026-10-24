#![feature(no_core,lang_items,rustc_attrs,const_trait_impl,const_destruct,auto_traits,allow_internal_unsafe,allow_internal_unstable)]
#![no_core]
#[lang="pointee_sized"] pub trait PointeeSized {}
#[lang="meta_sized"] pub trait MetaSized:PointeeSized {}
#[lang="sized"] pub trait Sized:MetaSized {}
#[lang = "destruct"]
#[diagnostic::on_unimplemented(message = "can't drop `{Self}`")]
#[rustc_deny_explicit_impl]
#[rustc_dyn_incompatible_trait]
pub const trait Destruct: PointeeSized {}
#[lang = "freeze"]
pub unsafe auto trait Freeze {}
#[lang="clone"] pub trait Clone:Sized { #[lang="clone_fn"] fn clone(&self)->Self; }
#[lang="copy"] pub trait Copy:Clone {}
#[lang="legacy_receiver"] pub trait LegacyReceiver:PointeeSized {}
impl<T:PointeeSized> LegacyReceiver for &T {}
impl<T:PointeeSized> LegacyReceiver for &mut T {}
impl Clone for u32 { fn clone(&self)->Self { *self } }
impl Copy for u32 {}
impl Clone for u8 { fn clone(&self)->Self { *self } }
impl Copy for u8 {}
#[rustc_builtin_macro]
macro_rules! concat_bytes { ($($tt:tt)*) => {{ /* compiler built-in */ }}; }
#[unsafe(no_mangle)] pub extern "C" fn answer()->*const u8 { let _=concat_bytes!(b"AB");let _=concat_bytes!(b"AB");let _=concat_bytes!(b"AB");let _=concat_bytes!(b"AB");let _=concat_bytes!(b"AB");let _=concat_bytes!(b"AB");let _=concat_bytes!(b"AB");concat_bytes!(b"AB") as *const [u8;2] as *const u8 }
