fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
extern crate first;extern crate second;fn main(){unsafe{first::MUT=41;}check(second::increment()==42);check(second::read()==36);}