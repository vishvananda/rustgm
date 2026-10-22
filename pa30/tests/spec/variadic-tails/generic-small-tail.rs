unsafe extern "C"{fn ignores_tail(tag:u32,...)->u32;}
fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn call<T>(value:T)->u32{unsafe{ignores_tail(3,value)}}fn main(){check(call(7u8)==3);}
