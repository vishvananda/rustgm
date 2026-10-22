unsafe extern "C"{fn ignores_tail(tag:u32,...)->u32;}
fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn main(){let x=7u64;check(unsafe{ignores_tail(3,&x,&x)}==3);}
