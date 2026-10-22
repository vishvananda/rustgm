#[derive(Copy,Clone)]#[repr(C)]enum E<T>{A(T),Empty}const fn round<T:Copy>(mut e:E<T>)->E<T>{unsafe{core::ptr::write((&raw mut e)as *mut u8,0)};e}const N:(u32,bool)=(match round(E::A(37u32)){E::A(v)=>v,_=>0},match round(E::A(true)){E::A(v)=>v,_=>false});fn main(){check(N.0==37&&N.1)}
fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
