fn check(v:bool){if !v{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn drive<R,F:FnOnce()->R>(f:F)->R{f()}
fn project<'a,F:FnOnce()->(&'a mut u8,)>(f:F)->&'a mut u8{drive(f).0}
fn main(){let mut n=1u8;let p=&mut n;let y=project(move||(p,));n=2;check(*y==2);}
