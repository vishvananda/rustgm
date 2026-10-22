unsafe extern "C"{fn sum(n:u32,...)->i64;}
fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn invoke<T>(_:T)->i64{unsafe{sum(2,3i64,5i64)}}fn main(){check(invoke(0u8)==8);check(invoke(0u16)==8);}
