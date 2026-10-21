fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn call<'a>(f:&dyn for<'b> Fn(&'b u8)->&'b u8,x:&'a u8)->&'a u8{f(x)} fn main(){let f:fn(&u8)->&u8=|x|x;let n=7;check(*call(&f,&n)==7);}
