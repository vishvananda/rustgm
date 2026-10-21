fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn call<'a,'b,F:FnOnce()->(&'a u8,&'b u8)>(f:F)->(&'a u8,&'b u8){f()}fn main(){let outer=7u8;let result;{let inner=9u8;let f=||(&inner,&outer);result=call(f).0;}check(*result==7);}
