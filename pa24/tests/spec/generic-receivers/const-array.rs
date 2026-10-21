fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn once<F:FnOnce()->R,R>(f:F)->R{f()}fn twice<F:FnMut()->R,R>(mut f:F)->R{let _=f();f()}fn take<const N:usize>(x:[u8;N])->u8{let f=move||x[0];once(f)}fn main(){check(take([7u8;2])==7);}
