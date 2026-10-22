fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn eq<T:PartialEq<U>,U>(a:&[T],b:&[U])->bool{a==b}fn main(){check(eq(&[1u8,2],&[1u8,2]));}
