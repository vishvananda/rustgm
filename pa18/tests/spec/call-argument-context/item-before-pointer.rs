fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn same<T>(_:T,_:T){}fn id(x:u8)->u8{x}fn main(){let f:fn(u8)->u8=id;same(id,f);}
