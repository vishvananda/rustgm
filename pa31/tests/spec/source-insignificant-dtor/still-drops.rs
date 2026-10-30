#![feature(rustc_attrs)]
#![allow(internal_features,dead_code)]
#[rustc_insignificant_dtor]
struct Marked<'a>(&'a mut u8);
impl Drop for Marked<'_>{fn drop(&mut self){*self.0=3;}}
struct Payload<'a>(&'a mut u8);
impl Drop for Payload<'_>{fn drop(&mut self){*self.0=7;}}
#[rustc_insignificant_dtor]
struct Wrapper<'a>{value:Payload<'a>}
#[unsafe(no_mangle)]pub extern "C" fn observed()->u8{
 let mut first=0;let mut second=0;
 {let _own=Marked(&mut first);let _field=Wrapper{value:Payload(&mut second)};}
 if first==3&&second==7{37}else{0}
}
