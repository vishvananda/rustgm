fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}
fn f(mut n:u8)->u8{let mut sum=0;'outer:while n>0{let x;x=n;n-=1;if x==2{continue 'outer}sum+=x;}sum}fn main(){check(f(4)==8);}
