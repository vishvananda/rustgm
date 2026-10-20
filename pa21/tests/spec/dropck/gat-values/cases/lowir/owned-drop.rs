fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct D<'a>(&'a mut u8);impl<'a>Drop for D<'a>{fn drop(&mut self){*self.0+=1;}}trait F{type Out<'a>;fn make<'a>(x:&'a mut u8)->Self::Out<'a>;}struct S;impl F for S{type Out<'a>=D<'a>;fn make<'a>(x:&'a mut u8)->Self::Out<'a>{D(x)}}fn consume<T:F>(){let mut v=0u8;{let _d=T::make(&mut v);}check(v==1);}fn main(){consume::<S>();}
