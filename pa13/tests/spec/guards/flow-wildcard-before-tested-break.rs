fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}
fn entry()->u64{let mut x=7u8;loop{match x{_ if {x=8;break;}=>(),0=>(),_=>()}}x as u64}fn main(){check(entry()==8);}