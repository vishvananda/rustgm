fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn main(){let n=1u8;let f=move|x:&u8,y:&u8|n+*x+*y;let p:&dyn for<'a,'b>Fn(&'a u8,&'b u8)->u8=&f;let x=2;check(p(&x,&4)==7);}
