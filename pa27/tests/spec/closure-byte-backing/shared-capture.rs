#[repr(u8)]enum E<T>{A(T),Empty}static VALUE:u32=37;fn main(){let f=const{let r=&VALUE;let e=E::A(move||*r);match e{E::A(f)=>f,E::Empty=>loop{}}};check(f()==37)}
fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
