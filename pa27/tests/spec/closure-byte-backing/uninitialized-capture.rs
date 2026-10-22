union U<T:Copy>{f:T,b:u8}const fn broken<T:Copy>(_:T)->T{unsafe{U{b:1}.f}}fn main(){let f=const{let x=37u64;broken(move||x)};let _=f;}
fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
