fn check(v:bool){if !v{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn drive<R,F:FnOnce(u8)->R>(f:F)->R{f(1)}fn project<T,F:FnOnce(u16)->(T,)>(f:F)->T{drive(f).0}
fn main(){let _=project(|_:u16|(23u8,));}
