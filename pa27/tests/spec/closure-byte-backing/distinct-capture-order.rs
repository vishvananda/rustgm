#[repr(u8)]enum E<T>{A(T),Empty}fn main(){let f=const{let x=37u32;let y=5u32;let e=E::A(move||x*3+y);match e{E::A(f)=>f,E::Empty=>loop{}}};let g=const{let x=37u32;let y=5u32;let e=E::A(move||x+y*3);match e{E::A(g)=>g,E::Empty=>loop{}}};check(f()==116);check(g()==52)}
fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
