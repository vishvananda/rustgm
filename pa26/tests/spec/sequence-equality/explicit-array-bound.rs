fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn eq<T,U,const N:usize,const M:usize>(a:&[T;N],b:&[U;M])->bool where [T;N]:PartialEq<[U;M]>{a==b}fn main(){check(eq(&[1u8,2],&[1u8,2]));}
