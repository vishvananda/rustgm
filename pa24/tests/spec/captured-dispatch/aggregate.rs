fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn main(){let x=(7u8,9u16);let f=move|n:(u8,u16)|(x.0+n.0,x.1+n.1);let p:&dyn Fn((u8,u16))->(u8,u16)=&f;let y=p((2,3));check(y.0==9&&y.1==12);}
