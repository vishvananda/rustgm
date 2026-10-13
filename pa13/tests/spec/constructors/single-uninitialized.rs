fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}
enum E{A(u8)}fn main(){let e:E;match e{E::A(_)=>()}check(true);}