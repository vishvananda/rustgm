enum E{A(bool),B(u64),Empty}const N:bool=unsafe{let mut e=E::A(false);let p=match &mut e{E::A(v)=>v as *mut bool as *mut u8,_=>loop{}};e=E::Empty;core::ptr::write(p,233);match e{E::Empty=>true,_=>false}};fn main(){check(N)}
fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
