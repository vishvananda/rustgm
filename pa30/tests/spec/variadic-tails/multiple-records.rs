#[repr(C)]struct Pair{a:u64,b:f64}
unsafe extern "C"{fn pairs(tag:u32,...)->u32;}
fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn main(){check(unsafe{pairs(3,Pair{a:7,b:1.5},-9i32,Pair{a:11,b:2.5})}==1);}
