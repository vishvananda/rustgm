#![feature(no_core,lang_items,rustc_attrs,const_trait_impl,auto_traits)]#![no_core]
#[lang="pointee_sized"]pub trait PointeeSized{}
#[lang="meta_sized"]pub trait MetaSized:PointeeSized{}
#[lang="sized"]pub trait Sized:MetaSized{}
#[lang="legacy_receiver"]pub trait LegacyReceiver:PointeeSized{}
impl<T:PointeeSized> LegacyReceiver for &T{}
impl<T:PointeeSized> LegacyReceiver for &mut T{}
#[lang="clone"]#[rustc_trivial_field_reads]pub const trait Clone:Sized{#[lang="clone_fn"]fn clone(&self)->Self;fn additional(&self)->u8;}
#[lang="copy"]pub trait Copy:Clone{}
struct Q;fn bad(x:Q)->Q{let a=x;x}
