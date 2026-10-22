unsafe extern "C"{fn sum(n:u32,...)->i64;}
fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn main(){check(unsafe{sum(3,1i64,2i64,3i64)}==6);}
