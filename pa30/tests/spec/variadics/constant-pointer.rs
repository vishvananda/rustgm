unsafe extern "C"{fn sum(n:u32,...)->i64;}
fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
const P:unsafe extern "C" fn(u32,...)->i64=sum;fn main(){check(unsafe{P(2,7i64,9i64)}==16);}
