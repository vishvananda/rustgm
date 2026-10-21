fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn value<'a,T,const N:usize>(x:u8)->u8{let f=move||x;f()}fn main(){check(value::<u8,2>(7)==7);check(value::<u16,3>(9)==9);}
