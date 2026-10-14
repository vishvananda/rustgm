fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn a<T>(x:T,n:u8)->T{if n==0{x}else{b(x,n-1)}}fn b<T>(x:T,n:u8)->T{a(x,n)}fn main(){check(a(9u16,5)==9);}
