const P: *const u8 = &7;
const N: bool = if false { P == P } else { true };
fn f() -> bool {
    N
}
fn main() {}
