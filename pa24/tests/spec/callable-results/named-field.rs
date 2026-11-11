fn check(v:bool){if !v{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn drive<R,F:FnOnce()->R>(f:F)->R{f()}
struct Value<T>{v:T}fn project<T,F:FnOnce()->Value<T>>(f:F)->T{drive(f).v}
fn main(){check(project(||Value{v:23u8})==23);}
