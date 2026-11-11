fn check(v:bool){if !v{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn drive<R,F:FnOnce()->R>(f:F)->R{f()}fn project<T:Copy,F:FnOnce()->[T;2]>(f:F)->T{drive(f)[1]}
fn main(){check(project(||[17u8,23])==23);}
