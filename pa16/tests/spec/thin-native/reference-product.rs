fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn id<'a>(x:(&'a u8,&'a u8))->(&'a u8,&'a u8){x}fn main(){let a=7;let b=9;let r=id((&a,&b));check(*r.0==7 && *r.1==9);}
