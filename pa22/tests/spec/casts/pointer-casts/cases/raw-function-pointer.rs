fn f(p: *const u8) -> fn() {
    p as fn()
}
fn main() {}
