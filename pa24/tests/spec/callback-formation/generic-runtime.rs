trait Factory: Sized { fn make() -> Self; }
enum State<T, F> { Waiting(F), Ready(T) }
struct Lazy<T, F = fn() -> T> { state: State<T, F> }
impl<T, F: FnOnce() -> T> Lazy<T, F> { fn new(f: F) -> Self { Self { state: State::Waiting(f) } } }
fn build<T: Factory>() -> Lazy<T> { Lazy::new(T::make) }
fn main() {}
