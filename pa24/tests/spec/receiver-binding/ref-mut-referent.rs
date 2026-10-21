fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn main(){let ref mut r=7u8;let mut f=move||{*r+=1;*r};check(f()==8);check(f()==9);}
