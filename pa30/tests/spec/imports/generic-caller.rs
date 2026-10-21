unsafe extern "C" {fn imported_i32(x:i32)->i32;}
fn wrapper<T>(x:i32,_value:T)->i32{unsafe{imported_i32(x)}}
fn check(ok:bool){if !ok{let _=fail(0);}}
fn fail(x:u8)->u8{1/x}
fn main(){check(wrapper(39,())==42);}
