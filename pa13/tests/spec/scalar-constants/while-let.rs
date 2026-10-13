fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}
const N:u8=7;fn main(){let mut n=7u8;let mut c=0;while let N=n{c+=1;n=0;}check(c==1);}