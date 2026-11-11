#![feature(const_trait_impl)]
const trait Factory: Sized { fn make() -> Self; }
const impl Factory for u8 { fn make() -> Self { 41 } }
const fn invoke<T: [const] Factory>() -> T { T::make() }
const VALUE:u8=invoke::<u8>();
fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn main(){check(VALUE==41);}
