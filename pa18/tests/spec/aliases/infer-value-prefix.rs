fn fail(x: u8) -> u8 { 1 / x }
fn check(ok: bool) { if !ok { let _ = fail(0); } }
struct S<T> { value: T }
type A<T> = S<T>;
fn main() {
    let a = A { value: 517u16 };
    let b: A<u8> = A { value: 7 };
    check(a.value == 517);
    check(b.value == 7);
}
