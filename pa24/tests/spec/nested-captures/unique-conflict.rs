fn verify(ok:bool){if !ok{let _=fault(0);}}fn fault(x:u8)->u8{1/x}
fn main(){let mut x=7u8;let r=&mut x;let mut f=||{let mut g=||{let _=&r;*r+=1;};g();};let q=&r;f();verify(**q==8);}
