fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn main(){let f:fn(u8,u16)->u16=|x,y|x as u16+y;check(f(7,1000)==1007);}
