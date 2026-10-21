fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn main(){let mut r=&mut 7u8;let f=move||{let old=r;r=old;*r+=1;*r};check(f()==8);}
