fn check(v:bool){if !v{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn drive<A,R,F:FnOnce(A)->R>(f:F,a:A)->R{f(a)}fn project<F:for<'a>FnOnce(&'a u8)->(&'a u8,)>(f:F,x:&u8)->&u8{drive(f,x).0}fn identity(x:&u8)->(&u8,){(x,)}
fn main(){let n=23u8;check(*project(identity,&n)==23);}
