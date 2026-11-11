#![allow(dead_code,unused_variables)]
use core::ops::{Deref,DerefMut};
struct Hold<T>(T);
impl<T>Deref for Hold<T>{type Target=T;fn deref(&self)->&T{let _=tick(2);&self.0}}
impl<T>DerefMut for Hold<T>{fn deref_mut(&mut self)->&mut T{let _=tick(3);&mut self.0}}
unsafe extern "C"{safe fn tick(code:u8)->u8;safe fn observe(before:u8,after:u8)->u8;}
fn make(n:u8)->Hold<u8>{let _=tick(1);Hold(n)}
fn argument()->u8{tick(4)}
fn replace(x:&mut u8,n:u8)->u8{let old=*x;*x=n;old}
fn read(x:&u8)->u8{*x}
#[unsafe(no_mangle)]pub extern "C" fn observed_coercion(n:u8)->u8{
 let mut h=make(n);let old=replace(&mut h,argument());let new=read(&h);observe(old,new)
}
fn main(){}
