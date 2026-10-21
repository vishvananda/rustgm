fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn infer<'a,F:for<'b>Fn(&'b u8)->&'a u8>(f:F)->F{f}fn take<'a,T>(x:&'a u8)->&'a u8{let k=0u8;let f=infer(|v|{let _=k;v});f(x)}fn main(){let x=7;let _=take::<u8>(&x);}
