fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn main(){let a=[[1u8,2],[3,4]];let b=[[1u8,2],[3,5]];check(a!=b);check(a==a);}
