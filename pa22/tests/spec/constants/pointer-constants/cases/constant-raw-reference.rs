const R: &u8 = &7;
const P: *const u8 = &raw const *R;
fn f() -> *const u8 {
    P
}
fn main() {}
