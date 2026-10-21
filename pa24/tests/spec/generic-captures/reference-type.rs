fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn take<T:Copy>(x:T)->T{let f=move||x;f()}fn main(){let a=7u8;let r=take(&a);check(*r==7);}
