fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn side(x:&mut u64)->u8{*x+=1;7}fn f(x:&mut u64)->&'static [u8;0]{&[side(x);0]}fn main(){let mut n=0;let r=f(&mut n);match r{[]=>()}check(n==1);}
