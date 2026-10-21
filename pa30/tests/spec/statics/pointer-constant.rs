fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
unsafe extern "C"{#[link_name="provided_readonly"]static VALUE:u32;}
const POINTER:*const u32=&raw const VALUE;
fn main(){check(unsafe{*POINTER}==7);}
