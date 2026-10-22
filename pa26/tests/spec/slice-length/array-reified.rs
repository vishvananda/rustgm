fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn main(){let f:for<'a> fn(&'a [u8;3])->usize= <[u8;3]>::len;check(f(&[1,2,3])==3);}
