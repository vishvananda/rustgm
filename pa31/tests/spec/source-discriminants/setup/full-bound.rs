#![feature(no_core,lang_items,rustc_attrs,intrinsics,core_intrinsics,auto_traits,staged_api,const_trait_impl,const_destruct)]
#![no_core]
#![unstable(feature="discriminant_fixture",issue="none")]
#![allow(internal_features,dead_code)]
#[lang="pointee_sized"]pub trait PointeeSized{}
#[lang="meta_sized"]pub trait MetaSized:PointeeSized{}
#[lang="sized"]pub trait Sized:MetaSized{}
#[lang="legacy_receiver"]pub trait LegacyReceiver:PointeeSized{}
impl<T:PointeeSized>LegacyReceiver for &T{}
impl<T:PointeeSized>LegacyReceiver for &mut T{}
#[lang="clone"]pub trait Clone:Sized{#[lang="clone_fn"]fn clone(&self)->Self;}
#[lang="copy"]pub trait Copy:Clone{}
#[lang="freeze"]pub unsafe auto trait Freeze{}
#[unstable(feature="const_destruct",issue="133214")]
#[rustc_const_unstable(feature="const_destruct",issue="133214")]
#[lang="destruct"]
#[rustc_deny_explicit_impl]
#[rustc_dyn_incompatible_trait]
pub const trait Destruct:PointeeSized{}
// Required method signatures and Formatter storage copied from pinned core.
// This declaration reducer does not supply integer formatting/hashing implementations.
#[rustc_diagnostic_item="Send"]pub unsafe auto trait Send{}
#[lang="sync"]#[rustc_diagnostic_item="Sync"]pub unsafe auto trait Sync{}
#[lang="unpin"]pub auto trait Unpin{}
pub const trait PartialEq<Rhs:PointeeSized=Self>:PointeeSized{fn eq(&self,other:&Rhs)->bool;}
pub const trait Eq:[const]PartialEq<Self>+PointeeSized{}
pub enum Result<T,E>{Ok(T),Err(E)}
pub mod fmt{
 use super::{PointeeSized,Result as Outcome};
 pub struct Error;
 pub type Result=Outcome<(),Error>;
 pub trait Write{fn write_str(&mut self,s:&str)->Result;}
 pub struct FormattingOptions{flags:u32,width:u16,precision:u16}
 pub struct Formatter<'a>{options:FormattingOptions,buf:&'a mut(dyn Write+'a)}
 #[rustc_diagnostic_item="Debug"]pub trait Debug:PointeeSized{fn fmt(&self,f:&mut Formatter<'_>)->Result;}
}
pub trait Hasher{fn finish(&self)->u64;fn write(&mut self,bytes:&[u8]);}
pub trait Hash:PointeeSized{fn hash<H:Hasher>(&self,state:&mut H);}
use fmt::Debug;
// Original complete DiscriminantKind bound family; no bound is erased.
#[lang="discriminant_kind"]#[rustc_deny_explicit_impl]#[rustc_dyn_incompatible_trait]
pub trait DiscriminantKind { #[lang="discriminant_type"]type Discriminant:Clone+Copy+Debug+Eq+PartialEq+Hash+Send+Sync+Unpin; }
#[unstable(feature="core_intrinsics",issue="none")]
#[rustc_intrinsic_const_stable_indirect]#[rustc_nounwind]#[rustc_intrinsic]
pub const fn discriminant_value<T>(v:&T)-><T as DiscriminantKind>::Discriminant;
