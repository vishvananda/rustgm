fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct D<'a,const B:bool,const C:char,const N:i8>(&'a mut u8);impl<const K:i8,const H:char,const A:bool> Drop for D<'_,A,H,K>{fn drop(&mut self){*self.0+=if A{1}else{2};*self.0+=H as u8;*self.0+=if K == -7{1}else{2};}}fn main(){let mut n=0;{let _d=D::<true,'A',{-7}>(&mut n);}check(n==67);}
