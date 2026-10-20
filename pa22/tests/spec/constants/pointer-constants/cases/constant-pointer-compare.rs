const P: *const u8 = &7;
const B: bool = P == P;
fn f() -> bool {
    B
}
fn main() {}
