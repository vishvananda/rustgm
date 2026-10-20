const P: *const u8 = ("abc" as *const str) as *const u8;
fn f() -> *const u8 {
    P
}
fn main() {}
