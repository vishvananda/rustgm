fn verify(ok:bool){if !ok{let _=fault(0);}}fn fault(x:u8)->u8{1/x}
fn main(){let x:u8;let f=||{let g=||{let _=x;};g();};x=7;f();}
