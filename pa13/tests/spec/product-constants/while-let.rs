fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}
const C:(u8,bool)=(7,true);fn main(){let mut x=(7u8,true);let mut n=0;while let C=x{n+=1;x=(0,false);}check(n==1);}