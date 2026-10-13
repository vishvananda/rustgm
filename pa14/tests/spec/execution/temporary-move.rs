fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}
struct N(u8);fn take(x:N)->u8{x.0}fn f()->u8{let x=(N(2),N(7)).1;take(x)}fn main(){check(f()==7);}
