#[repr(u8)]enum E<T>{A(T),Empty}fn main(){let f=const{let z=();let e=E::A(move||{let _=z;37u32});match e{E::A(f)=>f,E::Empty=>loop{}}};check(f()==37)}
fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
