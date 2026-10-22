fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn length<T,const N:usize>(a:&[T;N])->usize{a.len()}fn main(){check(length(&[1u8,2,3])==3);check(length::<u8,0>(&[])==0);}
