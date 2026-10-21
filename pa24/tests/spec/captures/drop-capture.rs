fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct D<'a>(&'a mut u8);impl Drop for D<'_>{fn drop(&mut self){*self.0+=2;}}fn consume<T>(_:T){}fn main(){let mut n=7u8;{let d=D(&mut n);let _f=move ||consume(d);}check(n==9);}
