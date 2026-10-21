fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn once<F:FnOnce()->R,R>(f:F)->R{f()}fn twice<F:FnMut()->R,R>(mut f:F)->R{let _=f();f()}fn take<'a,T:Copy>(x:T,y:&'a u8)->u8{let f=move||{let _v=x;*y};once(f)}fn main(){let y=9;check(take(&7u32,&y)==9);}
