fn check(v:bool){if !v{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn drive<R,F:FnOnce()->R>(f:F)->R{f()}
trait Kind{type Out;}impl Kind for u8{type Out=u8;}
fn take<'a,T:Kind>(x:&'a T::Out)->&'a T::Out{let f=move||(x,);drive(f).0}
fn main(){let n=23u8;check(*take::<u8>(&n)==23);}
