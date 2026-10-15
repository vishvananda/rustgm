fn fail(x: u8) -> u8 { 1 / x }
fn check(ok: bool) { if !ok { let _ = fail(0); } }
struct S<const N: usize>([u8; N]);
fn last<const N: usize>(s: S<N>) -> u8 { s.0[N - 1] }
fn read<const N: usize>(s: &S<N>, i: usize) -> u8 { s.0[i] }
fn main() {
    check(last(S([2, 3, 7])) == 7);
    check(read(&S([4, 5]), 1) == 5);
}
