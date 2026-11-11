fn check(v:bool){if !v{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn drive<R,F:FnOnce()->R>(f:F)->R{f()}
fn project<F:FnOnce()->(u8,)>(f:F)->u16{drive(f).0}
fn main(){let _=project(||(23u8,));}
