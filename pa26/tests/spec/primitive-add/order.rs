fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn add<T:core::ops::Add<Output=T>>(a:T,b:T)->T{a+b}fn mark(x:&mut u8)->u8{*x+=1;*x}fn main(){let mut n=0;check(add(mark(&mut n),mark(&mut n))==3);check(n==2);}
