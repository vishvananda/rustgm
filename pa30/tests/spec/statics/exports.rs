fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
#[repr(C)]struct Pair{a:u32,b:f64}
#[unsafe(no_mangle)]static ffi_readonly:u32=7;
#[unsafe(export_name="ffi_mutable")]static mut MUTABLE:u32=9;
#[unsafe(export_name="ffi_pair")]static PAIR:Pair=Pair{a:7,b:2.5};
unsafe extern "C"{safe fn read_exports()->u32;safe fn change_export();safe fn read_pair()->f64;}
fn main(){check(read_exports()==16);change_export();check(unsafe{MUTABLE}==12);check(read_pair()==9.5);}
