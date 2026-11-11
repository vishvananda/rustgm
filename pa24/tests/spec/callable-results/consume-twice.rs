fn check(v:bool){if !v{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn drive<R,F:FnOnce()->R>(f:F)->R{f()}
struct Value(u8);fn project<T,F:FnOnce()->(T,)>(f:F)->T{let _=drive(f).0;drive(f).0}
fn main(){let v=Value(23);let _=project(move||(v,));}
