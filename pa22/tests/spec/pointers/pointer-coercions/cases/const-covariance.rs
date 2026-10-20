fn f<'a, 'b: 'a>(p: *const &'b u8) -> *const &'a u8 {
    p
}
fn main() {}
