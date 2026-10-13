fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}
struct N(u8);fn main(){let x=[N(1)];match x{[_,n@..] if {let[a]=x;let _=a;let _=n;true}=>(),_=>()}}