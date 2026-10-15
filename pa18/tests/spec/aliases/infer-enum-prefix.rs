fn fail(x: u8) -> u8 { 1 / x }
fn check(ok: bool) { if !ok { let _ = fail(0); } }
enum E<T, const N: usize> { V(T, [u8; N]) }
type A<T = u8, const N: usize = 1> = E<T, N>;
fn main() {
    let E::V(x, a) = A::V(517u16, [3, 7]);
    check(x == 517);
    check(a[1] == 7);
}
