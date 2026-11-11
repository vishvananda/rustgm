#![feature(const_trait_impl)]
const trait Factory: Sized { fn make() -> Self; }
impl Factory for u8 { fn make() -> Self { 23 } }
const fn initializer<T: Factory>() -> fn() -> T { T::make }
const INIT: fn() -> u8 = initializer::<u8>();
fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn main(){check(INIT()==23);}
