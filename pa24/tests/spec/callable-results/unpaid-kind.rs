fn check(v:bool){if !v{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn drive<R,F:FnMut()->R>(mut f:F)->R{f()}fn project<T,F:FnOnce()->(T,)>(f:F)->T{drive(f).0}
fn main(){let _=project(||(23u8,));}
