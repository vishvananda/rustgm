fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn shared<F:Fn()->u8>(f:F)->u8{f()}fn mutable<F:FnMut()->u8>(mut f:F)->u8{f()}fn once<F:FnOnce()->u8>(f:F)->u8{f()}
struct D<'a>{value:u8,drops:&'a mut u8}impl Drop for D<'_>{fn drop(&mut self){*self.drops+=1;}}fn main(){let mut drops=0u8;{let d=D{value:7,drops:&mut drops};let f=move||{let _keep=&d;d.value};let r=&f;check(once(&r)==7);check(f()==7);}check(drops==1);}
