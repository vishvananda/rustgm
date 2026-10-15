enum E<T, U> { V(T, U) }
type A<T, U> = E<T, U>;
fn main() { let _ = A::<u8>::V(7, 9u16); }
