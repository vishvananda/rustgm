#![feature(no_core,lang_items,const_trait_impl,rustc_attrs,auto_traits)]
#![no_core]
#![allow(internal_features,dead_code)]
#[lang="pointee_sized"]pub trait PointeeSized{}
#[lang="meta_sized"]pub trait MetaSized:PointeeSized{}
#[lang="sized"]pub trait Sized:MetaSized{}
#[lang="freeze"]pub unsafe auto trait Freeze{}
#[lang="destruct"]#[rustc_deny_explicit_impl]#[rustc_dyn_incompatible_trait]pub const trait Destruct:PointeeSized{}
#[lang="legacy_receiver"]pub trait LegacyReceiver:PointeeSized{}
impl<T:PointeeSized>LegacyReceiver for &T{}
impl<T:PointeeSized>LegacyReceiver for &mut T{}
#[lang="pin"]#[repr(transparent)]pub struct Pin<P>{pointer:P}
impl<P>Pin<P>{#[lang="new_unchecked"]pub const unsafe fn new_unchecked(pointer:P)->Self{Self{pointer}}}
#[lang="deref"]pub trait Deref:PointeeSized{#[lang="deref_target"]type Target:PointeeSized;fn deref(&self)->&Self::Target;}
impl<T:PointeeSized>Deref for Pin<&mut T>{type Target=T;fn deref(&self)->&T{&*self.pointer}}
impl<T:PointeeSized>LegacyReceiver for Pin<&mut T>{}
#[lang="clone"]pub trait Clone:Sized{#[lang="clone_fn"]fn clone(&self)->Self;}
#[lang="copy"]pub trait Copy:Clone{}
impl Clone for u8{fn clone(&self)->Self{*self}}
impl Copy for u8{}
#[lang="Poll"]pub enum Poll<T>{#[lang="Ready"]Ready(T),#[lang="Pending"]Pending}
#[lang="Context"]pub struct Context<'a>{token:&'a ()}
#[lang="future_trait"]pub trait Future{
#[lang="future_output"]type Output;
#[lang="poll"]fn poll(self:Pin<&mut Self>,cx:&mut Context<'_>)->Poll<Self::Output>;
}
pub fn poll_item<F:Future>()->for<'a,'b,'c>fn(Pin<&'a mut F>,&'b mut Context<'c>)->Poll<F::Output>{F::poll}
fn context()->Context<'static>{Context{token:&()}}
pub async fn value()->u8{3}
#[unsafe(no_mangle)]pub extern "C" fn observe()->u8{
    let mut future=value();
    let mut cx=context();
    match Future::poll(unsafe{Pin::new_unchecked(&mut future)},&mut cx){
        Poll::Ready(value)=>value,
        Poll::Pending=>loop{},
    }
}
fn main(){}
