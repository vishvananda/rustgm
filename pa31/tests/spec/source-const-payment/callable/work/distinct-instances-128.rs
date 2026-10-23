#![feature(unboxed_closures)]
#![feature(no_core,lang_items,rustc_attrs,const_trait_impl)]
#![no_core]
#![allow(internal_features,dead_code,unused_variables)]
#[lang="pointee_sized"]pub trait PointeeSized{}
#[lang="meta_sized"]pub trait MetaSized:PointeeSized{}
#[lang="sized"]pub trait Sized:MetaSized{}
#[lang="legacy_receiver"]pub trait LegacyReceiver:PointeeSized{}
impl<T:PointeeSized> LegacyReceiver for &T{}
impl<T:PointeeSized> LegacyReceiver for &mut T{}
#[lang="clone"]pub const trait Clone:Sized{#[lang="clone_fn"]fn clone(&self)->Self;}
#[lang="copy"]pub trait Copy:Clone{}
impl Clone for u8{fn clone(&self)->Self{*self}}impl Copy for u8{}
#[lang="tuple_trait"]#[rustc_deny_explicit_impl]pub trait Tuple{}
#[lang="fn_once"]pub const trait FnOnce<Args:Tuple>{#[lang="fn_once_output"]type Output;extern "rust-call" fn call_once(self,args:Args)->Self::Output;}
#[lang="fn_mut"]pub const trait FnMut<Args:Tuple>:FnOnce<Args>{extern "rust-call" fn call_mut(&mut self,args:Args)->Self::Output;}
#[lang="fn"]pub const trait Fn<Args:Tuple>:[const]FnMut<Args>{extern "rust-call" fn call(&self,args:Args)->Self::Output;}
const fn need<F:const FnOnce<()>>(f:&F)->u8{7}const fn item<const K:usize>()->u8{1}#[unsafe(no_mangle)]pub extern "C" fn answer()->u8{need(&item::<0>);need(&item::<1>);need(&item::<2>);need(&item::<3>);need(&item::<4>);need(&item::<5>);need(&item::<6>);need(&item::<7>);need(&item::<8>);need(&item::<9>);need(&item::<10>);need(&item::<11>);need(&item::<12>);need(&item::<13>);need(&item::<14>);need(&item::<15>);need(&item::<16>);need(&item::<17>);need(&item::<18>);need(&item::<19>);need(&item::<20>);need(&item::<21>);need(&item::<22>);need(&item::<23>);need(&item::<24>);need(&item::<25>);need(&item::<26>);need(&item::<27>);need(&item::<28>);need(&item::<29>);need(&item::<30>);need(&item::<31>);need(&item::<32>);need(&item::<33>);need(&item::<34>);need(&item::<35>);need(&item::<36>);need(&item::<37>);need(&item::<38>);need(&item::<39>);need(&item::<40>);need(&item::<41>);need(&item::<42>);need(&item::<43>);need(&item::<44>);need(&item::<45>);need(&item::<46>);need(&item::<47>);need(&item::<48>);need(&item::<49>);need(&item::<50>);need(&item::<51>);need(&item::<52>);need(&item::<53>);need(&item::<54>);need(&item::<55>);need(&item::<56>);need(&item::<57>);need(&item::<58>);need(&item::<59>);need(&item::<60>);need(&item::<61>);need(&item::<62>);need(&item::<63>);need(&item::<64>);need(&item::<65>);need(&item::<66>);need(&item::<67>);need(&item::<68>);need(&item::<69>);need(&item::<70>);need(&item::<71>);need(&item::<72>);need(&item::<73>);need(&item::<74>);need(&item::<75>);need(&item::<76>);need(&item::<77>);need(&item::<78>);need(&item::<79>);need(&item::<80>);need(&item::<81>);need(&item::<82>);need(&item::<83>);need(&item::<84>);need(&item::<85>);need(&item::<86>);need(&item::<87>);need(&item::<88>);need(&item::<89>);need(&item::<90>);need(&item::<91>);need(&item::<92>);need(&item::<93>);need(&item::<94>);need(&item::<95>);need(&item::<96>);need(&item::<97>);need(&item::<98>);need(&item::<99>);need(&item::<100>);need(&item::<101>);need(&item::<102>);need(&item::<103>);need(&item::<104>);need(&item::<105>);need(&item::<106>);need(&item::<107>);need(&item::<108>);need(&item::<109>);need(&item::<110>);need(&item::<111>);need(&item::<112>);need(&item::<113>);need(&item::<114>);need(&item::<115>);need(&item::<116>);need(&item::<117>);need(&item::<118>);need(&item::<119>);need(&item::<120>);need(&item::<121>);need(&item::<122>);need(&item::<123>);need(&item::<124>);need(&item::<125>);need(&item::<126>);need(&item::<127>)}
