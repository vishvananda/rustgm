trait Factory: Sized { fn make() -> Self; }
struct Slot<T> { f: fn() -> T }
impl<T> Slot<T> { fn new(f: fn() -> T) -> Self { Self { f } } }
fn build<T: Factory>() -> Slot<T> { Slot::new(T::make) }
fn main() {}
