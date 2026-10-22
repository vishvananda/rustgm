#[repr(u8)]enum E<T>{A(T),Empty}fn main(){let mut f=const{let mut x=0u32;let e=E::A(move||{x+=1;x});match e{E::A(f)=>f,E::Empty=>loop{}}};check(f()==1);check(f()==2)}
fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
