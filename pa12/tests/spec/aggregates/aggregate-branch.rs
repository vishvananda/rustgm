fn check(ok:bool){if !ok{let _=failure(0);}}
fn failure(x:i32)->i32{1/x}
struct S(u64,u8);fn make(b:bool)->S{if b{S(7,3)}else{S(11,5)}}fn main(){let a=make(true);let b=make(false);check(a.0==7 && a.1==3 && b.0==11 && b.1==5);}
