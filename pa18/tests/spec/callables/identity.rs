fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn id<T>(x:T)->T{x}fn main(){check(id(7u8)==7);check(id(300u16)==300);check(id(9u8)==9);}
