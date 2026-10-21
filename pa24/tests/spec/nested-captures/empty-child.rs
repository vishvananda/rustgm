fn verify(ok:bool){if !ok{let _=fault(0);}}fn fault(x:u8)->u8{1/x}
fn main(){let f=||{let g=||7u8;g()};verify(f()==7);}
