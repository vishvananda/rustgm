fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait Apply:Fn(u8)->u8{} impl<F:Fn(u8)->u8>Apply for F{} fn run(f:&dyn Apply)->u8{f(3)} fn main(){let n=4;let f=|x:u8|x+n;check(run(&f)==7);}
