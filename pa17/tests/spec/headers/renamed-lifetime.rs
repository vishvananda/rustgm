struct S<'a>(&'a mut u8);impl<'b> Drop for S<'b>{fn drop(&mut self){*self.0+=1;}}fn main(){}
