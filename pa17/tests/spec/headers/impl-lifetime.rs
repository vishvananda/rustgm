struct S<'a>(&'a mut u8);impl<'a> Drop for S<'a>{fn drop(&mut self){*self.0+=1;}}fn main(){}
