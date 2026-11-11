fn check(v:bool){if !v{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn drive<R,F:FnOnce()->R>(f:F)->R{f()}
fn project()->&'static u8{let n=23u8;drive(||(&n,)).0}
fn main(){let _=project();}
