fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn get(x:&[u8],i:usize)->u8{x[i]}fn main(){check(get(&[3,7,9],1)==7);}
