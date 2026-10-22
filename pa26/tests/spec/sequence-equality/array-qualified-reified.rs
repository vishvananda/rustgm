fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn main(){let a=[1u8,2];let b=[1u8,2];let f:fn(&[u8;2],&[u8;2])->bool=PartialEq::eq;check(f(&a,&b));check(<[u8;2] as PartialEq>::eq(&a,&b));}
