#![feature(const_trait_impl)]
const trait Factory: Sized { fn make() -> Self; }
const fn build<T>() -> fn() -> T { T::make }
fn main() {}
