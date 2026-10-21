fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn identity<'a>(x:&'a u8)->&'a u8{x}fn main(){let n=1u8;let f=move|g:for<'a>fn(&'a u8)->&'a u8,x:&u8|n+*g(x);let p:&dyn for<'b>Fn(for<'a>fn(&'a u8)->&'a u8,&'b u8)->u8=&f;check(p(identity,&7)==8);}
