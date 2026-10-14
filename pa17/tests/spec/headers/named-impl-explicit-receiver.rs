struct S<'a>(&'a mut u8);impl<'b> Drop for S<'b>{fn drop<'r>(self:&'r mut Self){*self.0+=1;}}fn main(){}
