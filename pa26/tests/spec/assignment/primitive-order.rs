fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn place<'a>(x:&'a mut u32,n:&mut u32)->&'a mut u32{*n=*n*10+1;x}fn rhs(n:&mut u32)->u32{*n=*n*10+2;4}fn main(){let mut c=3u32;let mut n=0;*place(&mut c,&mut n)+=rhs(&mut n);check(c==7&&n==21);}
