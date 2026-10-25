const fn read(x:&u8)->u8{*x}
type Early=[u8;read(&7u8) as usize];
#[unsafe(no_mangle)]pub extern "C" fn answer()->u8{let _:Early=[0u8;7];7}
#[unsafe(no_mangle)]pub extern "C" fn runtime()->u8{read(&7u8)}
