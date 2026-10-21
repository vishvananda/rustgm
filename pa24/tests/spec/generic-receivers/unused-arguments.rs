fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn once<F:FnOnce()->R,R>(f:F)->R{f()}fn twice<F:FnMut()->R,R>(mut f:F)->R{let _=f();f()}fn take<T,const N:usize>(x:u8)->u8{let f=move||x;once(f)}fn main(){check(take::<u8,1>(7)==7);check(take::<u16,2>(8)==8);}
