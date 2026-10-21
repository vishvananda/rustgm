fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct T(u8);fn eat(x:T)->u8{x.0}fn main(){let x=T(7);let f=move||eat(x);let _: &dyn FnOnce()->u8=&f;}
