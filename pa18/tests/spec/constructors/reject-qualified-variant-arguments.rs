enum E<T> { V(T) }
fn main() { let _ = <E<u8>>::V::<u8>(7); }
