fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}
struct N{n:u8}fn f(x:(N,u8))->u8{let (n,_)=x;match x{(_,v)=>n.n+v}}fn main(){check(f((N{n:7},9))==16);}