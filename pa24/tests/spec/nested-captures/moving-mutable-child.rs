fn verify(ok:bool){if !ok{let _=fault(0);}}fn fault(x:u8)->u8{1/x}
fn main(){let mut x=7u8;let f=||{let mut g=move||{x+=1;x};g()};verify(f()==8);verify(f()==8);verify(x==7);}
