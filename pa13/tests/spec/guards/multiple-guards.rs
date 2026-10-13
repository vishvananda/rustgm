fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}
fn f(x:u8)->u8{match x{n if n==0=>10,n if n==1=>20,n if n==2=>30,n=>n}}fn main(){check(f(0)==10&&f(1)==20&&f(2)==30&&f(7)==7);}