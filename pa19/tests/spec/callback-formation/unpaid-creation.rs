trait Factory: Sized { fn make() -> Self; }
fn build<T>() -> fn() -> T { T::make }
fn main() {}
