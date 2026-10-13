fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn f(s:&[u8],i:usize)->u8{s[i]}fn main(){let _=f(&[],0);}
