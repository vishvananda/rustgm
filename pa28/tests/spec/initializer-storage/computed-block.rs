const fn word(x:u8)->u8{x}static DATA:&u8=&{let mut x=word(7);x=9;x};
#[unsafe(no_mangle)]pub extern "C" fn observed()->u8{*DATA}
