struct D<'a>(&'a mut u8);impl Drop for D<'_>{fn drop(&mut self){*self.0+=1;}}fn observe(_:u8){}fn f(mut n:u8){let mut a=0;while n>0{{let d=D(&mut a);}observe(a);n-=1;}}fn main(){}
