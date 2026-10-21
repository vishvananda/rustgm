fn verify(ok:bool){if !ok{let _=fault(0);}}fn fault(x:u8)->u8{1/x}
fn main(){let x=3u8;let f=||{let x=7u8;let g=||x;g()};verify(f()==7);verify(x==3);}
