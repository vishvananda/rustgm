fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}
struct N;fn main(){let x=[N];match x{[_,n@..] if {let[a]=x;let _=a;true}=>{let _=n;},_=>()}}