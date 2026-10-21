unsafe extern "C" {
    safe fn invoke_callback(callback:extern "C" fn(i32)->i32,x:i32)->i32;
}
extern "C" fn twice(x:i32)->i32{x*2}
fn check(ok:bool){if !ok{let _=fail(0);}}
fn fail(x:u8)->u8{1/x}
fn main(){check(invoke_callback(twice,20)==42);}
