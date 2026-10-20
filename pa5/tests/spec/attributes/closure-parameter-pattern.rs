fn main() {
    let f = |#[allow(unused)] (left, right): (u8, u8)| left + right;
    let _ = f((2, 3));
}
