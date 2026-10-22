const fn make()->impl Copy {true}
union U<T:Copy>{a:T,b:bool}
const N:bool={let mut x=make();unsafe{*(&raw mut x as *mut u8)=2;U{a:x}.b}};
fn main(){let _=N;}
