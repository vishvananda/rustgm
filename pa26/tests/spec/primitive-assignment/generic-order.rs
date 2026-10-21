fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn place<'a,T>(x:&'a mut T,n:&mut u32)->&'a mut T{*n=*n*10+1;x}fn rhs(n:&mut u32)->u32{*n=*n*10+2;4}fn apply<T:core::ops::AddAssign<u32>>(x:&mut T,n:&mut u32){*place(x,n)+=rhs(n);}fn main(){let mut x=3u32;let mut n=0;apply(&mut x,&mut n);check(x==7&&n==12);}
