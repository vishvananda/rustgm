fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct Count<'a>(&'a mut u8);struct Size(usize);type D<'a,T=Count<'a>,const N:usize={Size(3).0}>=T;impl<'a> Drop for D<'a>{fn drop(&mut self){*self.0+=1;}}fn consume<'a>(_:Count<'a>){}fn main(){let mut n=0;consume(Count(&mut n));check(n==1);}
