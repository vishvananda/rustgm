fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn eq<T:PartialEq<U>,U,const N:usize>(a:&[T;N],b:&[U;N])->bool{a==b}fn main(){check(eq(&[1u8,2],&[1u8,2]));check(eq::<u8,u8,0>(&[],&[]));}
