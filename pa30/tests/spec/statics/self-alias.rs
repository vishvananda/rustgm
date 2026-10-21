#[unsafe(export_name="ffi_x")]static mut X:u32=7;unsafe extern "C"{#[link_name="ffi_x"]static mut Y:u32;}fn main(){unsafe{let _=Y;X=9;}}
