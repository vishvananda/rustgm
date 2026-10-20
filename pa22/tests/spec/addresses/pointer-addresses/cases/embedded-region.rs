fn f<'a>(x: &'a u8) -> *const &'static u8 {
    &raw const x
}
fn main() {}
