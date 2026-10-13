fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn side(x:&mut u64){*x+=1}fn f(x:&mut u64)->&'static u8{&{side(x);7}}fn main(){let mut n=0;check(*f(&mut n)==7 && n==1);}
