#![feature(const_trait_impl)]
const trait Factory: Sized { fn make() -> Self; }
enum State<T, F> { Waiting(F), Ready(T) }
struct Lazy<T, F = fn() -> T> { state: State<T, F> }
impl<T, F: FnOnce() -> T> Lazy<T, F> { const fn new(f: F) -> Self { Self { state: State::Waiting(f) } } }
const impl<T: Factory> Factory for Lazy<T> { fn make() -> Self { let init: fn() -> T = T::make; Lazy::new(init) } }
fn main() {}
