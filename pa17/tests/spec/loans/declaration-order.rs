struct D<'a>(&'a mut u8);impl Drop for D<'_>{fn drop(&mut self){*self.0+=1;}}fn observe(_:u8){}fn f(){let d;let mut a=0;d=D(&mut a);}fn main(){}
