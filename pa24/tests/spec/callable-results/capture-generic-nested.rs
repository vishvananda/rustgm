fn check(v:bool){if !v{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn drive<R,F:FnOnce()->R>(f:F)->R{f()}
fn take<'a,T>(x:&'a T)->&'a T{let f=move||(x,);{let r=&f;drive(&r).0}}
fn main(){let n=23u8;check(*take(&n)==23);}
