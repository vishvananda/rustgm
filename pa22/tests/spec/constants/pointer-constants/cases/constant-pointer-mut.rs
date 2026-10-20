const P: *mut u8 = &7u8 as *const u8 as *mut u8;
fn f() -> *mut u8 {
    P
}
fn main() {}
