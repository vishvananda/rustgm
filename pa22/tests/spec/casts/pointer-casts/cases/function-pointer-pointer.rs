fn f(g: fn()) -> *const u8 {
    g as *const u8
}
fn main() {}
