fn f<T: ?Sized>(p: *const T) -> *const u8 {
    p as *const u8
}
fn main() {}
