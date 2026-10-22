unsafe extern "C"{fn sum(n:u32,...)->i64;}
fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn next(x:&mut i64)->i64{*x+=1;*x}fn main(){let mut n=0i64;check(unsafe{sum(3,next(&mut n),next(&mut n),next(&mut n))}==6);check(n==3);}
