#![feature(no_core,lang_items,intrinsics,rustc_attrs,const_trait_impl,const_destruct)]
#![no_core]
#![allow(internal_features,dead_code,unused_variables)]
#[lang="pointee_sized"]pub trait PointeeSized{}
#[lang="meta_sized"]pub trait MetaSized:PointeeSized{}
#[lang="sized"]pub trait Sized:MetaSized{}
#[lang="legacy_receiver"]pub trait LegacyReceiver:PointeeSized{}
impl<T:PointeeSized> LegacyReceiver for &T{}
#[lang="clone"]pub trait Clone:Sized{#[lang="clone_fn"]fn clone(&self)->Self;}
#[lang="copy"]pub trait Copy:Clone{}
#[lang="destruct"]#[rustc_deny_explicit_impl]#[rustc_dyn_incompatible_trait]pub const trait Destruct:PointeeSized{}
#[lang="drop_glue"]pub const unsafe fn drop_glue<T:PointeeSized>(_:&mut T)where T:[const]Destruct{}
#[rustc_intrinsic]pub const fn type_name<T:?Sized>()->&'static str;
/// As core's `any::type_name`, through an inline constant of a generic function.
pub const fn name_of<T:?Sized>()->&'static str{const{type_name::<T>()}}
pub struct Pair<A,B>(A,B);
pub struct Holder<'a>(&'a u8);
pub struct Mixed<'a,T>(&'a T);
pub enum Option<T>{None,Some(T)}
pub struct Stack<T,const N:usize>([T;N]);
pub struct Signed<const N:i8>;
pub struct Flag<const B:bool>;
pub mod inner{
    pub struct Wrapper<T:?crate::Sized>(pub *const T);
    pub mod deeper{pub struct Leaf;}
}
#[unsafe(no_mangle)]pub static mut NAMES:[&str;26]=[
    type_name::<u8>(),
    type_name::<bool>(),
    type_name::<char>(),
    type_name::<i128>(),
    type_name::<f64>(),
    type_name::<str>(),
    type_name::<()>(),
    type_name::<(u8,)>(),
    type_name::<(u8,i8,bool)>(),
    type_name::<[u8;4]>(),
    type_name::<[u16]>(),
    type_name::<&u8>(),
    type_name::<&'static mut [u8]>(),
    type_name::<*const u8>(),
    type_name::<*mut (u8,bool)>(),
    type_name::<&&str>(),
    type_name::<Pair<u8,i8>>(),
    type_name::<inner::Wrapper<&'static str>>(),
    type_name::<Holder<'static>>(),
    type_name::<Option<Pair<u8,u8>>>(),
    type_name::<Stack<u8,4>>(),
    type_name::<Signed<{0xFDu8 as i8}>>(),
    type_name::<Flag<true>>(),
    type_name::<[Option<u8>;2]>(),
    type_name::<inner::deeper::Leaf>(),
    type_name::<Mixed<'static,u8>>(),
];
#[unsafe(no_mangle)]pub static mut GENERIC:&str=name_of::<Pair<u8,i8>>();
#[unsafe(no_mangle)]pub static mut FILLED:&str="";
/// The inline constant of a function instance that runs.
#[unsafe(no_mangle)]pub extern "C" fn fill(){unsafe{FILLED=name_of::<Option<u16>>();}}
/// A function pointer, which rustc names `fn(u8) -> u8` and this printer does not spell.
#[unsafe(no_mangle)]pub extern "C" fn refused()->usize{let name=name_of::<fn(u8)->u8>();0}
