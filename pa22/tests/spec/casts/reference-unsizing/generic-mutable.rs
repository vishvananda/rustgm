fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn cast<T,const N:usize>(p:&mut [T;N])->&mut [T]{p as &mut [T]}fn main(){let mut a=[3u8,5];let s=cast(&mut a);s[0]=8;check(a[0]==8);}
