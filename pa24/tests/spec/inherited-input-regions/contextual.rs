pub fn f<'b,'c>(x:&'c &'b u8)->&'b u8{let f:fn(&'c &'b u8)->&'b u8=|&s|s;f(x)}
fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn main(){let x=7u8;let p=&x;let r=f(&p);check(*r==7);}
