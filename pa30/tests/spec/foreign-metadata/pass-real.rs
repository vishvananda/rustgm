fn main(){check(one::real(1.25,2.5)==3.75);}
fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(n:u8)->u8{1/n}

