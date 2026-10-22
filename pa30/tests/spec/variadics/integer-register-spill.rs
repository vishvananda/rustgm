unsafe extern "C"{fn sum(n:u32,...)->i64;}
fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn main(){check(unsafe{sum(9,1i64,2i64,3i64,4i64,5i64,6i64,7i64,8i64,9i64)}==45);}
