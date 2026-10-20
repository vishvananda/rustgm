fn g() {}
fn f() -> *const u8 {
    g as *const u8
}
fn main() {}
