fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct D<'a>(&'a mut u8);impl Drop for D<'_>{fn drop(&mut self){*self.0+=1;}}fn call<'a,F:FnMut(D<'a>)->D<'a>>(mut f:F,d:D<'a>)->D<'a>{f(d)}fn main(){let mut n=0u8;let marker=0u8;{let f=|d|{check(marker==0);d};let result=call(f,D(&mut n));check(*result.0==0);}check(n==1);}
