fn check(ok:bool){if !ok{let _=failure(0);}} fn failure(x:i32)->i32{1/x}
#[repr(C,u8)]enum E{A(u64,u8),B{a:u128}} fn id(x:E)->E{x} fn main(){let a=id(E::A(42,7));let b=id(E::B{a:99});let _=(a,b);}
