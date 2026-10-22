fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn same<T:?Sized+PartialEq<U>,U:?Sized>(a:&T,b:&U)->bool{a==b}fn main(){check(same(&7u8,&7u8));}
