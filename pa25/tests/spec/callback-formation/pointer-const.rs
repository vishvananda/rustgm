trait Factory: Sized { fn make() -> Self; }
struct Slot<T> { f: fn() -> T }
impl<T> Slot<T> { const fn new(f: fn() -> T) -> Self { Self { f } } }
const fn build<T: Factory>() -> Slot<T> { Slot::new(T::make) }
fn main() {}
