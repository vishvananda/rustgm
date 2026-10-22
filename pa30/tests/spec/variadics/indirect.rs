unsafe extern "C"{fn sum(n:u32,...)->i64;}
fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn invoke(p:unsafe extern "C" fn(u32,...)->i64)->i64{unsafe{p(2,11i64,13i64)}}fn main(){check(invoke(sum)==24);}
