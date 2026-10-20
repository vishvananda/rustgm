fn main() {
    let f = |#[cfg(false)] gone: Missing, value: u8| value + 1;
    let _ = f(2);
}
