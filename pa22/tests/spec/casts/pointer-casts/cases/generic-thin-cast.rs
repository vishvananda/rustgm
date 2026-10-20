fn f<T>(p: *const T) -> *const u8 {
    p as *const u8
}
fn main() {}
