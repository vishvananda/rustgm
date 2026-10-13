fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}
struct N(u8);fn main(){let mut n=0u8;let mut sum=0;while n<3{let xs=[N(n),N(5)];let[a,b]=xs;sum+=a.0+b.0;n+=1;}check(sum==18);}