union U{bits:u8,value:bool} const X:u32={let source=U{bits:2};let mut array=[(false,10u32),(true,20),(false,30)];let [_,ref mut rest @ ..]=array;unsafe{core::ptr::copy(&raw const source.value,&raw mut rest[0].0,1);}rest[0].1+=2;rest[0].0=true;array[1].1};
fn main(){check(X==22);}
