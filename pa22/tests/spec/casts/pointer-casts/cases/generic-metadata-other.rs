fn f<T: ?Sized, U: ?Sized>(p: *const T) -> *const U {
    p as *const U
}
fn main() {}
