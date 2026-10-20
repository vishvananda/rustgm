fn f<'a>(s: &'a &'a [u8; 2]) -> &'a [u8] {
    s
}
fn main() {}
