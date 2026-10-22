#[repr(u8)]enum E<T>{A(T),Empty}fn main(){let f=const{let x=37u32;let e=E::A(move||x+1);match e{E::A(f)=>f,E::Empty=>loop{}}};let g=const{let x=37u32;let e=E::A(move||x+9);match e{E::A(g)=>g,E::Empty=>loop{}}};check(f()==38);check(g()==46)}
fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
