fn check(v:bool){if !v{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn drive<R,F:FnOnce()->R>(f:F)->R{f()}
fn project<'a,F:FnOnce()->(&'a u8,)>(f:F)->&'static u8{drive(f).0}
fn main(){let n=23u8;let _=project(||(&n,));}
