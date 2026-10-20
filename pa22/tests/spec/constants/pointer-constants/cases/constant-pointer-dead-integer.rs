const P: *const u8 = &7;
const N: usize = if false { P as usize } else { 0 };
fn f() -> usize {
    N
}
fn main() {}
