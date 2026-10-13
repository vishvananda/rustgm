fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}
fn f(x:u8,y:bool)->u8{match x{0=>match y{true=>7,false=>9},_=>3}}fn main(){check(f(0,true)==7&&f(0,false)==9&&f(1,true)==3);}