fn f<T: ?Sized>(p: *const u8) -> *const T {
    p as *const T
}
fn main() {}
