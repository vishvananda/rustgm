unsafe extern "C"{fn sum_tail(n:u32,...)->i64;}
fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn main(){let f:unsafe extern "C" fn(_,...) -> _=sum_tail;check(unsafe{f(2,4i64,5i64)}==9);}
