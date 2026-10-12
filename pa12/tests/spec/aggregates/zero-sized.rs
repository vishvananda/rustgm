fn check(ok:bool){if !ok{let _=failure(0);}} fn failure(x:i32)->i32{1/x}
#[derive(Clone,Copy)]struct Z; fn id(x:[Z;18446744073709551615])->[Z;18446744073709551615]{x} fn main(){let a=id([Z;18446744073709551615]);let _=a[8];check(true);}
