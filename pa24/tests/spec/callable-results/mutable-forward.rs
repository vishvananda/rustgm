fn check(v:bool){if !v{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn drive<R,F:FnOnce()->R>(f:F)->R{f()}
fn project<T,F:FnMut()->(T,)>(mut f:F)->T{drive(&mut f).0}
fn main(){let mut n=0u8;check(project(||{n+=1;(23u8,)})==23);check(n==1);}
