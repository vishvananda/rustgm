fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn make(n:u8)->impl Fn(u8)->u8{move |x|x+n}fn main(){let f=make(7);check(f(2)==9);}
