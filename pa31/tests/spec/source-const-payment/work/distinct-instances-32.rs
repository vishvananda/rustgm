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
const impl Clone for u8{fn clone(&self)->Self{*self}}impl Copy for u8{}
const fn unused<T:[const]Clone,const K:usize>()->u8{7}const N:u8={unused::<u8,0>();unused::<u8,1>();unused::<u8,2>();unused::<u8,3>();unused::<u8,4>();unused::<u8,5>();unused::<u8,6>();unused::<u8,7>();unused::<u8,8>();unused::<u8,9>();unused::<u8,10>();unused::<u8,11>();unused::<u8,12>();unused::<u8,13>();unused::<u8,14>();unused::<u8,15>();unused::<u8,16>();unused::<u8,17>();unused::<u8,18>();unused::<u8,19>();unused::<u8,20>();unused::<u8,21>();unused::<u8,22>();unused::<u8,23>();unused::<u8,24>();unused::<u8,25>();unused::<u8,26>();unused::<u8,27>();unused::<u8,28>();unused::<u8,29>();unused::<u8,30>();unused::<u8,31>()};#[unsafe(no_mangle)]pub extern "C" fn answer()->u8{N}
