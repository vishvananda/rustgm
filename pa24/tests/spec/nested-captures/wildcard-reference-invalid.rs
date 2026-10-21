fn verify(ok:bool){if !ok{let _=fault(0);}}fn fault(x:u8)->u8{1/x}
fn main(){let mut x=7u8;let p=(&mut x,);x=9;let f=||{let g=||{let _=p.0;};g();};f();}
