fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn run<F:Fn(u8)->u8+?Sized>(f:&F)->u8{f(3)}fn main(){let n=4;let f=|x|x+n;let g:&dyn Fn(u8)->u8=&f;check(run(g)==7);}
