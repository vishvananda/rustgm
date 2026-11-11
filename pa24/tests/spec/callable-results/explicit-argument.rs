fn check(v:bool){if !v{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn drive<R,F:FnOnce()->R>(f:F)->R{f()}
fn project<T,F:FnOnce()->(T,)>(f:F)->T{drive::<(T,),_>(f).0}
fn main(){check(project(||(23u8,))==23);}
