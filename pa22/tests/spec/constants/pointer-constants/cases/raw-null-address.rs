const P: *const u8 = 0usize as *const u8;
const Q: *const u8 = &raw const *P;
fn f() -> *const u8 {
    Q
}
fn main() {}
