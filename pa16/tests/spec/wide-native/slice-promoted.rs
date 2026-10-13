fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn f()->&'static [u8]{&[7,9]}fn main(){check(f()[1]==9);}
