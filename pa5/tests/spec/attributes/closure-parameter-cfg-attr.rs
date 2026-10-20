fn main() {
    let f = move |#[cfg_attr(true, cfg(false))] gone: Missing, value: u8| -> u8 { value };
    let _ = f(2);
}
