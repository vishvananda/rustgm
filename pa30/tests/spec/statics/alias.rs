fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
#[unsafe(export_name="ffi_self")]static mut LOCAL:u32=7;
unsafe extern "C"{#[link_name="ffi_self"]static mut OTHER:u32;}
fn main(){let p=&raw mut LOCAL;let q=&raw mut OTHER;check(p==q);unsafe{*q=11;check(LOCAL==11);}}
