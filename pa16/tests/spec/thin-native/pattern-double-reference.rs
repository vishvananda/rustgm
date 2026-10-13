fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn f(x:&&(u8,u8))->u8{match x{&&(a,b)=>a+b}}fn main(){check(f(&&(3,4))==7);}
