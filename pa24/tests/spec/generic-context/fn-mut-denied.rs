fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn infer<F:for<'a>Fn(&'a u8)->&'a u8>(f:F)->F{f}fn main(){let mut n=0u8;let _f=infer(|x|{n+=1;x});}
