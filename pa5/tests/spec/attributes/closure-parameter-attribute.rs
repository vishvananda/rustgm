fn main() {
    let f = |#[allow(unused)] value: u8| value + 1;
    let _ = f(2);
}
