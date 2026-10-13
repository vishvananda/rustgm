fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn inc(x:&mut u64){*x+=1}fn main(){let mut x=40;let r=&mut x;inc(r);inc(r);check(*r==42);}
