fn bump(x:&mut u8){*x+=1;}struct S<'a>(&'a mut u8);impl Drop for S<'_>{fn drop(&mut self){bump(self.0);}}fn main(){}
