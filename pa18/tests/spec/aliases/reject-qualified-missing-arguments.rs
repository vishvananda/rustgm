enum E<T> { V(T) }
type A<T> = E<T>;
fn main() { let _ = <A>::V(7u8); }
