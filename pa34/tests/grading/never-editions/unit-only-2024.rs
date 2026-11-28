#![feature(never_type)]
#![allow(dependency_on_unit_never_type_fallback)]
enum Maybe<T>{None,Some(T)}
trait Tag{fn tag()->u8;}
impl Tag for (){fn tag()->u8{21}}
fn consume<T:Tag>(_:Maybe<T>)->u8{T::tag()}
fn probe()->u8{let x=if true{Maybe::None}else{Maybe::Some(return 0)};consume(x)}
fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn main(){check(probe()==24);}
