unsafe extern "C"{fn sum(n:u32,args:...)->i64;}
fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn main(){check(unsafe{sum(1,23i64)}==23);}
