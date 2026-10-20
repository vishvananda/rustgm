fn f<'a, 'b: 'a>(p: *mut &'b u8) -> *mut &'a u8 {
    p
}
fn main() {}
