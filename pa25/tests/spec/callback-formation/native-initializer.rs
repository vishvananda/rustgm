#![feature(const_trait_impl)]
const trait Factory: Sized { fn make() -> Self; }
enum State<T, F> { Waiting(F), Ready(T) }
struct Lazy<T, F = fn() -> T> { state: State<T, F> }
impl<T, F: FnOnce() -> T> Lazy<T, F> { const fn new(f: F) -> Self { Self { state: State::Waiting(f) } } }
const impl<T: Factory> Factory for Lazy<T> { fn make() -> Self { Lazy::new(T::make) } }
impl Factory for u8 { fn make() -> Self { 23 } }
fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn main(){let cell=<Lazy<u8> as Factory>::make();match cell.state{State::Waiting(f)=>check(f()==23),State::Ready(_)=>check(false)}}
