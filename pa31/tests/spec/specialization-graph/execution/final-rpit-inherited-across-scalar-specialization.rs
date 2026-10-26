#![allow(dead_code,unused_imports,incomplete_features,internal_features,unused_macros)]
#![feature(specialization)]
trait Get{fn get(self)->u8;}impl Get for u8{fn get(self)->u8{self}}
trait V{fn f()->impl Get+Copy;fn g()->u8;}impl<T>V for T{fn f()->impl Get+Copy{17u8}default fn g()->u8{1}}impl V for u16{fn g()->u8{2}}
fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn main(){check(<u16 as V>::f().get()==17);check(<u16 as V>::g()==2);}
