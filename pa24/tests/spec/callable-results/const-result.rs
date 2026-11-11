fn check(v:bool){if !v{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn drive<R,F:FnOnce()->R>(f:F)->R{f()}
fn project<const N:usize,F:FnOnce()->([u8;N],)>(f:F)->[u8;N]{drive(f).0}
fn main(){check(project(||([17u8,23],))[1]==23);}
