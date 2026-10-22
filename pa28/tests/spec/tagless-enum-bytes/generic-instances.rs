#[derive(Copy,Clone)]enum E<T>{Only(T)}const fn round<T:Copy>(mut e:E<T>,b:u8)->E<T>{unsafe{core::ptr::write((&raw mut e)as *mut u8,b)};e}const N:(u32,bool)=(match round(E::Only(37u32),37){E::Only(v)=>v},match round(E::Only(true),1){E::Only(v)=>v});fn main(){check(N.0==37&&N.1)}
fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
