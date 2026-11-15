#![allow(dead_code,unused_variables)]
pub fn make<T>(output:&T)->Option<&T>{Some(output)}
pub fn choose<T>(input:Option<u8>,output:&T)->Option<&T>{input?;make(output)}
#[unsafe(no_mangle)]pub extern "C" fn entry()->u8{let value=41u8;match choose(Some(0),&value){Some(v)=>*v,None=>23}}
#[unsafe(no_mangle)]pub extern "C" fn probe(value:u8)->u8{match choose(Some(0),&value){Some(v)=>*v,None=>23}}
#[unsafe(no_mangle)]pub extern "C" fn fail()->u8{let value=59u8;match choose(None,&value){Some(v)=>*v,None=>23}}
fn main(){}
