fn check(v:bool){if !v{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn drive<R,F:FnOnce()->R>(f:F)->R{f()}
fn value()->u8{23}fn project<F:FnOnce()->(fn()->u8,)>(f:F)->u8{(drive(f).0)()}
fn main(){check(project(||(value as fn()->u8,))==23);}
