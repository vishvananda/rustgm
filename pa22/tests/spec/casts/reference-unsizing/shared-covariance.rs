fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn cast<'a,'b>(p:&'b [&'a u8;1])->&'b [&'b u8] where 'a:'b{p as &'b [&'b u8]}fn main(){let x=7;let a=[&x];check(*cast(&a)[0]==7);}
