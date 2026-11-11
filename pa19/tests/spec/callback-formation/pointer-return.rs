trait Factory: Sized { fn make() -> Self; }
fn build<T: Factory>() -> fn() -> T { T::make }
fn main() {}
