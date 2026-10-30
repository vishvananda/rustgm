#![feature(rustdoc_internals)]
#![feature(no_core,lang_items,rustc_attrs,const_trait_impl)]
#![feature(staged_api)]
#![stable(feature="from_provider",since="1.0.0")]
#![no_core]
#![allow(internal_features,dead_code,unused_variables)]
#[stable(feature="from_provider",since="1.0.0")]
#[lang="pointee_sized"]pub trait PointeeSized{}
#[stable(feature="from_provider",since="1.0.0")]
#[lang="meta_sized"]pub trait MetaSized:PointeeSized{}
#[stable(feature="from_provider",since="1.0.0")]
#[lang="sized"]pub trait Sized:MetaSized{}
#[rustc_diagnostic_item = "From"]
#[lang = "From"]
#[stable(feature = "rust1", since = "1.0.0")]
#[rustc_on_unimplemented(on(
    all(Self = "&str", T = "alloc::string::String"),
    note = "to coerce a `{T}` into a `{Self}`, use `&*` as a prefix",
))]
#[doc(search_unbox)]
#[rustc_const_unstable(feature = "const_convert", issue = "143773")]
pub const trait From<T>: Sized {
    /// Converts to this type from the input type.
    #[rustc_diagnostic_item = "from_fn"]
    #[must_use]
    #[stable(feature = "rust1", since = "1.0.0")]
    fn from(value: T) -> Self;
}
#[stable(feature="from_provider",since="1.0.0")]impl From<f32> for f32{fn from(value:f32)->Self{32.0}}
#[stable(feature="from_provider",since="1.0.0")]impl From<f64> for f32{fn from(value:f64)->Self{64.0}}
#[stable(feature="from_provider",since="1.0.0")]#[unsafe(no_mangle)]pub extern "C" fn observed()->u8{<f32 as From<_>>::from(1.0) as u8}
#[stable(feature="from_provider",since="1.0.0")]#[lang="legacy_receiver"]pub trait LegacyReceiver:PointeeSized{}
#[stable(feature="from_provider",since="1.0.0")]impl<T:?Sized>LegacyReceiver for &T{}
#[stable(feature="from_provider",since="1.0.0")]#[lang="clone"]pub trait Clone:Sized{#[stable(feature="from_provider",since="1.0.0")]#[lang="clone_fn"]fn clone(&self)->Self;}
#[stable(feature="from_provider",since="1.0.0")]#[lang="copy"]pub trait Copy:Clone{}
#[stable(feature="from_provider",since="1.0.0")]impl Clone for f32{fn clone(&self)->Self{*self}}
#[stable(feature="from_provider",since="1.0.0")]impl Copy for f32{}
#[stable(feature="from_provider",since="1.0.0")]impl Clone for f64{fn clone(&self)->Self{*self}}
#[stable(feature="from_provider",since="1.0.0")]impl Copy for f64{}
