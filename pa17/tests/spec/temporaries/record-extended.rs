struct D<'a>(&'a u8);impl Drop for D<'_>{fn drop(&mut self){observe(*self.0);}}fn observe(_:u8){}fn make()->u8{3}fn f(){let d=D{0:&make()};}fn main(){}
