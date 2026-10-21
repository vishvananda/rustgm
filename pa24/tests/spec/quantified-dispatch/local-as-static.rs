fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn main(){let n=1u8;let p:&dyn for<'a>Fn(&'a u8)->&'static u8=&move|x|if n>0{x}else{x};let _=p;}
