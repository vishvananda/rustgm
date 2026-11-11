#![feature(const_trait_impl)]
const trait Factory: Sized { fn make() -> Self; }
const fn helper<T: [const] Factory>() -> T { T::make() }
const fn build<T: [const] Factory>() -> T { helper::<T>() }
fn main() {}
