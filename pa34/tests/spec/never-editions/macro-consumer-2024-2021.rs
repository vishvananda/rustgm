#![feature(never_type)]
#![allow(dependency_on_unit_never_type_fallback)]
enum Maybe<T>{None,Some(T)}
trait Tag{fn tag()->u8;}
impl Tag for (){fn tag()->u8{21}}
impl Tag for !{fn tag()->u8{24}}
fn consume<T:Tag>(_:Maybe<T>)->u8{T::tag()}
macros::make!();
fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn main(){check(probe()==21);}
