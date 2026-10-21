unsafe extern "C" {
    fn imported_i32(x:i32)->i32;
    safe fn imported_f64(x:f64)->f64;
}
fn check(ok:bool){if !ok{let _=fail(0);}}
fn fail(x:u8)->u8{1/x}
fn main(){check(unsafe{imported_i32(39)}==42);check(imported_f64(1.25)==3.75);}
