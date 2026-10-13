fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}
struct N(u8);fn f(mut count:u8)->u8{let mut x=N(1);while count>0{let y=x;x=N(y.0+1);count-=1;}x.0}fn main(){check(f(0)==1&&f(4)==5);}
