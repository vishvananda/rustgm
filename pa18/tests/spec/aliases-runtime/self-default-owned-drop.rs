fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct S<'a>(&'a mut u8);impl<'a> Drop for S<'a>{fn drop(&mut self){*self.0+=1;}}type A<'a,T=A<'a,S<'a>>>=T;fn consume<'a>(_:A<'a>){}fn main(){let mut n=0;consume(S(&mut n));check(n==1);}
