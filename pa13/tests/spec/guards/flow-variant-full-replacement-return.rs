fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}
enum E{A(u8),B}fn entry()->u64{let mut x=E::A(7);match x{E::A(_) if {x=E::B;return 7;}=>99,_=>98}}fn main(){check(entry()==7);}