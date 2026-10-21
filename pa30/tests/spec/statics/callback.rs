fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn id<T:Copy>(x:T)->T{x}
extern "C" fn callback(x:u32)->u32{id(x)+3}
#[unsafe(export_name="ffi_callback")]static CALLBACK:extern "C" fn(u32)->u32=callback;
unsafe extern "C"{safe fn invoke_exported(x:u32)->u32;}
fn main(){check(invoke_exported(39)==42);}
