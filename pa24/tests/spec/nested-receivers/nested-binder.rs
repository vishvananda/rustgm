fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn shared<F:Fn()->u8>(f:F)->u8{f()}fn mutable<F:FnMut()->u8>(mut f:F)->u8{f()}fn once<F:FnOnce()->u8>(f:F)->u8{f()}
fn context<F:for<'a> Fn(&'a u8)->&'a u8>(f:F)->F{f}fn invoke<F:for<'a> FnOnce(&'a u8)->&'a u8>(f:F,x:&u8)->&u8{f(x)}fn main(){let n=9u8;let f=context(|x|{let _keep=n;x});let r=&f;let x=7;check(*invoke(&r,&x)==7);}
