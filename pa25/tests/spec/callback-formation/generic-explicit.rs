trait Factory: Sized { fn make() -> Self; }
enum State<T, F> { Waiting(F), Ready(T) }
struct Lazy<T, F = fn() -> T> { state: State<T, F> }
impl<T, F: FnOnce() -> T> Lazy<T, F> { const fn new(f: F) -> Self { Self { state: State::Waiting(f) } } }
const fn build<T: Factory>() -> Lazy<T> { Lazy::<T, fn() -> T>::new(T::make) }
fn main() {}
