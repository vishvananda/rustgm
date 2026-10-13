fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}
#[repr(u64)]enum E{A=0,B=18446744073709551615}fn f(e:E)->u8{match e{E::A=>1,E::B=>2}}fn main(){check(f(E::B)==2);}