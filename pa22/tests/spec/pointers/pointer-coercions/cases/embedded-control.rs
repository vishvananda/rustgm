fn f<'a>(x: &'a u8) -> *const &'a u8 {
    &x
}
fn main() {}
