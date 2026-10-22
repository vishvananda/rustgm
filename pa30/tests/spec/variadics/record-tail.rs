#[repr(C)]struct Pair{a:u64,b:f64}unsafe extern "C"{fn record_tail(tag:u32,...)->u32;}
fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn main(){check(unsafe{record_tail(0,Pair{a:7,b:2.5})}==1);}
