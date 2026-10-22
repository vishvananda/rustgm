fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn pass<T>(n:&mut u8,x:T)->T{*n+=1;x}fn id(x:u8)->u8{x+1}fn main(){let mut n=0u8;check(pass(&mut n,id)(3)==4);check(n==1);}
