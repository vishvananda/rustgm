fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}
struct N(u8);fn f(x:(N,bool))->u8{if let (n,true)=x{n.0}else{x.0.0+1}}fn main(){check(f((N(7),true))==7&&f((N(8),false))==9);}