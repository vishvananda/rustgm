fn check(v:bool){if !v{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn invoke<A,B,R,F:FnOnce(A,B)->R>(f:F,a:A,b:B)->R{f(a,b)}fn project<'a,'b,F:for<'x,'y>FnOnce(&'x u8,&'y u8)->(&'y u8,)>(f:F,a:&'a u8,b:&'b u8)->&'a u8{invoke(f,a,b).0}fn second<'a,'b>(_:&'a u8,b:&'b u8)->(&'b u8,){(b,)}fn main(){let a=1u8;let b=2u8;let _=project(second,&a,&b);}
