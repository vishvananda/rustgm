#![feature(const_trait_impl)]
const trait Factory: Sized { fn make() -> Self; }
const fn build<T: [const] Factory>() -> T { let f: fn() -> T = T::make; f() }
fn main() {}
