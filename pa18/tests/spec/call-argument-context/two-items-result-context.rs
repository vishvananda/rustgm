fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn choose<T>(a:T,_:T)->T{a}fn id(x:u8)->u8{x}fn other(x:u8)->u8{x+1}fn main(){let f:fn(u8)->u8=choose(id,other);check(f(3)==3);}
