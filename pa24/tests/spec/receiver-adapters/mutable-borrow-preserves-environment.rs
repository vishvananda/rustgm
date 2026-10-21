fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct D<'a>(&'a mut u8);impl Drop for D<'_>{fn drop(&mut self){*self.0+=1;}}fn call<F:FnMut()->u8>(mut f:F)->u8{f()}fn main(){let mut n=6u8;{let d=D(&mut n);let mut f=move||{let _=&d;7u8};check(call(&mut f)==7);check(f()==7);}check(n==7);}
