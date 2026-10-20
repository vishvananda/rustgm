fn f<'a>(p: *const &'a u8) -> *const &'static u8 {
    p as *const &'static u8
}
fn main() {}
