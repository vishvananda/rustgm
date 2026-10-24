struct Pair(u32, u32);
#[unsafe(no_mangle)] pub extern "C" fn answer() -> u32 {
    let pair = Pair(19, 24);
    pair.0 + pair.1
}
