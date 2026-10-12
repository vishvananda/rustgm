fn check(ok:bool){if !ok{let _=failure(0);}}
fn failure(x:i32)->i32{1/x}
fn inc(x:u8)->u8{x+1}fn dec(x:u8)->u8{x-1}fn main(){let a:[fn(u8)->u8;2]=[inc,dec];check(a[0](4)==5 && a[1](4)==3);}
