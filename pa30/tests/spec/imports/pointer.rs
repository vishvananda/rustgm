unsafe extern {
    fn imported_i32(x:i32)->i32;
}
fn check(ok:bool){if !ok{let _=fail(0);}}
fn fail(x:u8)->u8{1/x}
fn main(){let p:unsafe extern "C" fn(i32)->i32=imported_i32;check(unsafe{p(39)}==42);}
