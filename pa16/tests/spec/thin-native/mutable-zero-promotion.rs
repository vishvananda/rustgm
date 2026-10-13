fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn f()->&'static mut [u8;0]{&mut []}fn main(){match f(){[]=>()}}
