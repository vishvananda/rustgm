fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
const F:fn(&[u8])->usize= <[u8]>::len;const N:usize=F(&[1]);fn main(){let _=N;}
