fn f<'a>(x: &'a u8) {
    let _a: [*const &'static u8; 0] = [&x; 0];
}
fn main() {}
