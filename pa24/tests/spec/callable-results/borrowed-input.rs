fn check(v:bool){if !v{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn drive<A,R,F:FnOnce(A)->R>(f:F,a:A)->R{f(a)}fn project<'a,F:FnOnce(&'a u8)->(&'a u8,)>(f:F,x:&'a u8)->&'a u8{drive(f,x).0}
fn main(){let n=23u8;check(*project(|x|(x,),&n)==23);}
