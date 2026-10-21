unsafe extern "C" {fn imported_i32(x:i32)->i32;}
const IMPORTED:unsafe extern "C" fn(i32)->i32=imported_i32;
fn check(ok:bool){if !ok{let _=fail(0);}}
fn fail(x:u8)->u8{1/x}
fn main(){check(unsafe{IMPORTED(39)}==42);}
