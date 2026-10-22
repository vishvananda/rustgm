#[repr(u8)]enum E<T>{A(T),Empty}static VALUE:u32=37;fn main(){let f=const{let r=&VALUE;let a=[1u8,2];let b=true;let e=E::A(move||if b{*r+a[0]as u32+a[1]as u32}else{0});match e{E::A(f)=>f,E::Empty=>loop{}}};check(f()==40)}
fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
