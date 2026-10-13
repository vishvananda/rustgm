fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}
const N:u8=7;const LO:u8=0;const HI:u8=9;fn main(){let mut c=0;match 7u8{N|LO..=HI|_ if {c+=1;false}=>(),_=>()}check(c==3);}