#![feature(const_trait_impl)]
const trait Factory: Sized { fn make() -> Self; }
const fn build<T: Factory>() -> fn() -> T { T::make }
fn main() {}
