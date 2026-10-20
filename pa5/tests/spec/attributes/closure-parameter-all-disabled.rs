fn main() {
    let f = |#[cfg(false)] gone: Missing| 3u8;
    let _ = f();
}
