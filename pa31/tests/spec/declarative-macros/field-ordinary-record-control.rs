struct Record { value: u32 }
#[unsafe(no_mangle)] pub extern "C" fn answer() -> u32 {
    let value = 47u32;
    let record = Record { value };
    let Record { value } = record;
    value
}
