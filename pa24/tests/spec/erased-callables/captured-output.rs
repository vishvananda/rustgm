fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn run<'a>(f:&(dyn Fn()->&'a u8+'a))->&'a u8{f()} fn main(){let n=7;let f=||&n;check(*run(&f)==7);}
