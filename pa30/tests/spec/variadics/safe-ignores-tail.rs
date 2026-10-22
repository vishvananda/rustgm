unsafe extern "C"{safe fn ignores(n:u32,...)->u32;}
fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn main(){check(ignores(7,1i64,2.0f64)==7);}
