fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn f<T>(x:T,n:u8)->T{if n==0{x}else{f(x,n-1)}}fn main(){check(f(7u8,4)==7);}
