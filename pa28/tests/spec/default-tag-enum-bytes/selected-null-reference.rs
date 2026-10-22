static S:u32=37;enum E{A(&'static u32),B(u64),Empty}const N:u32=unsafe{let mut e=E::A(&S);let p=match &mut e{E::A(v)=>v as *mut &u32 as *mut usize,_=>loop{}};core::ptr::write(p,0);match e{E::A(v)=>*v,_=>0}};fn main(){let _=N;}
fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
