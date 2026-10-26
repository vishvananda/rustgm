#![allow(dead_code,unused_imports,incomplete_features,internal_features,unused_macros)]
#![feature(min_specialization)]
trait Clone:Sized{fn clone(&self)->Self;}impl Clone for u8{fn clone(&self)->Self{*self}}
static mut HITS:u8=0;struct D(u8);impl Clone for D{fn clone(&self)->Self{D(self.0)}}impl Drop for D{fn drop(&mut self){unsafe{HITS+=self.0;}}}
trait W:Clone{fn go(&self,sink:&mut u8)->Self;fn danger()->u8;}
impl<T:Clone> W for T{default fn go(&self,sink:&mut u8)->Self{*sink+=1;self.clone()}default fn danger()->u8{fail(0)}}
impl W for u8{fn go(&self,sink:&mut u8)->Self{*sink+=10;*self}fn danger()->u8{7}}
fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn main(){let mut sink=0;{let x=D(2);let y=x.go(&mut sink);check(y.0==2);check(7u8.go(&mut sink)==7);}check(sink==11);check(unsafe{HITS}==4);check(<u8 as W>::danger()==7);}
