struct D<'a>(&'a mut u8);impl Drop for D<'_>{fn drop(&mut self){*self.0+=1;}}fn observe(_:u8){}fn f(){let mut a=0;{let d=D(&mut a);}let r=&a;a=2;observe(*r);}fn main(){}
