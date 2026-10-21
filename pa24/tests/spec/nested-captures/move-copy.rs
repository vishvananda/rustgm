fn verify(ok:bool){if !ok{let _=fault(0);}}fn fault(x:u8)->u8{1/x}
fn main(){let x=7u8;let f=move||{let g=move||x;g()};verify(f()==7);verify(f()==7);}
