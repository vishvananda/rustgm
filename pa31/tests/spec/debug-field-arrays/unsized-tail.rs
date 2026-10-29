#[derive(Debug)]
struct Tail<T: ?Sized> { a: u8, b: u8, c: u8, d: u8, e: u8, f: u8, tail: T }
fn require_unsized(value: &Tail<[u8]>) { let _ = format!("{:?}", value); }
