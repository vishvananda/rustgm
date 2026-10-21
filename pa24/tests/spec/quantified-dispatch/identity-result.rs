fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn main(){let n=1u8;let p:&dyn Fn(&u8)->&u8=&move|x|if n>0{x}else{x};let x=7u8;check(*p(&x)==7);}
