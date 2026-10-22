fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn eq<T:PartialEq<U>,U,const N:usize,const M:usize>(a:&[T;N],b:&[U;M])->bool{a==b}fn main(){}
