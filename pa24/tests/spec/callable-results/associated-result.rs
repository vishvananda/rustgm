fn check(v:bool){if !v{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn drive<R,F:FnOnce()->R>(f:F)->R{f()}
trait Kind{type Output;}struct K;impl Kind for K{type Output=u8;}fn project<T:Kind,F:FnOnce()->(T::Output,)>(f:F)->T::Output{drive(f).0}
fn main(){check(project::<K,_>(||(23u8,))==23);}
