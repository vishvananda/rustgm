fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn len<T,const N:usize>(_:&[T;N])->usize{N}fn main(){check(len(&[1u8,2,3])==3);}
