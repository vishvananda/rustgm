fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn main(){let mut n=1u8;let mut f=move|x:&u8|{n+=*x;n};let p:&mut dyn FnMut(&u8)->u8=&mut f;check(p(&2)==3);check(p(&4)==7);}
