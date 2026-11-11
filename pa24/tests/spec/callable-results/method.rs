fn check(v:bool){if !v{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct Owner;impl Owner{fn drive<R,F:FnOnce()->R>(&self,f:F)->R{f()}}fn project<T,F:FnOnce()->(T,)>(f:F)->T{Owner.drive(f).0}
fn main(){check(project(||(23u8,))==23);}
