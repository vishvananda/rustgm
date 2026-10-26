#![allow(dead_code,unused_imports,incomplete_features,internal_features,unused_macros)]
#![feature(specialization)]
trait Get{fn get(self)->u8;}impl Get for u8{fn get(self)->u8{self}}
struct W<T>(T);trait V{fn f()->impl Get+Copy;fn g()->u8;}
impl<T:'static>V for W<T>{fn f()->impl Get+Copy{17u8}default fn g()->u8{1}}
impl<'a>V for W<&'a u8>{fn g()->u8{2}}
fn use_it<'a>(_: &'a u8)->u8{<W<&'a u8> as V>::f().get()}
fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn main(){let x=7;check(use_it(&x)==17);}
