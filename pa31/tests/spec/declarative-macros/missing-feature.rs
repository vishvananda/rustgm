pub macro value() { 29u32 }
#[unsafe(no_mangle)] pub extern "C" fn answer() -> u32 { value!() }
