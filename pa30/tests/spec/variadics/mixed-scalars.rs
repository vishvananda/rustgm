unsafe extern "C"{fn mixed(tag:u32,...)->u32;}
fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn main(){let x=19u8;check(unsafe{mixed(0,-7i32,11u32,-13i64,17u64,2.5f64,&x as *const u8)}==1);}
