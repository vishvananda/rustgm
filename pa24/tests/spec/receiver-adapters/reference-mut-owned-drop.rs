fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct D<'a>{value:u8,drops:&'a mut u8}impl Drop for D<'_>{fn drop(&mut self){*self.drops+=1;}}fn call<F:FnOnce()->u8>(f:F)->u8{f()}fn main(){let mut drops=0u8;{let mut d=D{value:5,drops:&mut drops};let mut f=move||{let _=&d;d.value+=1;d.value};check(call(&mut f)==6);check(f()==7);}check(drops==1);}
