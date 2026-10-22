union U{bits:u8,value:bool} const X:bool={let source=U{bits:2};let mut target=[false;3];unsafe{core::ptr::copy(&raw const source.value,&raw mut target[0],1);}let [ref mut rest @ ..]=target;*rest=[true;3];target[0]};
fn main(){check(X);}
