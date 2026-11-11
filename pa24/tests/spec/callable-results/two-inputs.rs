fn check(v:bool){if !v{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn drive<A,B,R,F:FnMut(A,B)->R>(mut f:F,a:A,b:B)->R{f(a,b)}fn project<T,F:FnMut(u8,u16)->(T,)>(f:F)->T{drive(f,1u8,2u16).0}
fn main(){check(project(|_,_|(23u8,))==23);}
