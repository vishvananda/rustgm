fn verify(ok:bool){if !ok{let _=fault(0);}}fn fault(x:u8)->u8{1/x}
fn main(){let mut x=7u8;{let mut g=||{x+=1;};let r=&mut g;let mut f=||r();f();f();}verify(x==9);}
