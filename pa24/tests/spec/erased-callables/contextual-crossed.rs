fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn main(){let _f:&dyn for<'a,'b> Fn(&'a u8,&'b u8)->&'a u8=&|_,y|y;}
