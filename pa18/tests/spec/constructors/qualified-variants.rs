fn fail(x: u8) -> u8 { 1 / x }
fn check(ok: bool) { if !ok { let _ = fail(0); } }
enum E<T> { V(T) }
type A<T> = E<T>;
fn main() {
    let f: fn(u16) -> E<u16> = <A<u16>>::V;
    match f(517) { E::V(x) => check(x == 517), _ => check(false) }
    match <E<_>>::V(7u8) { E::V(x) => check(x == 7), _ => check(false) }
}
