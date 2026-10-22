#[repr(u8)]enum E<T>{A(T),Empty}fn main(){let f=const{let x=37u32;let r=&x;let e=E::A(move||*r);match e{E::A(f)=>f,E::Empty=>loop{}}};let _=f;}
fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
