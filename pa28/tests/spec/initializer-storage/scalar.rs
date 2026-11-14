const fn word(x:u8)->u8{x}static DATA:&u8=&word(7);#[unsafe(no_mangle)]pub extern "C" fn observed()->u8{*DATA}
