fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
enum E{A}fn f(x:&E)->u8{match x{E::A=>7}}fn main(){check(f(&E::A)==7);}
