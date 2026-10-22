fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
const fn length<T,const N:usize>(a:&[T;N])->usize{a.len()}const N:usize=length(&[1u8,2,3]);fn main(){check(N==3);}
