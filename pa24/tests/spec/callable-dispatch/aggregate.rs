fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn swap(x:(u8,u16))->(u16,u8){(x.1,x.0+2)}fn main(){let f:fn((u8,u16))->(u16,u8)=swap;let p:&dyn Fn((u8,u16))->(u16,u8)=&f;let v=p((7,1000));check(v.0==1000&&v.1==9);}
