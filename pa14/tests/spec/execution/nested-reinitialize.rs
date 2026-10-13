fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}
struct N(u8);fn f(mut x:((N,N),N))->u8{let a=x.0.0;let b=x.0.1;x.0=(N(a.0+1),N(b.0+1));x.0.0.0+x.0.1.0+x.1.0}fn main(){check(f(((N(2),N(3)),N(4)))==11);}
