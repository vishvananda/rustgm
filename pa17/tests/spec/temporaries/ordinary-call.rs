struct D<'a>(&'a u8);impl Drop for D<'_>{fn drop(&mut self){observe(*self.0);}}fn observe(_:u8){}fn make()->u8{3}fn wrap(x:&u8)->D<'_>{D(x)}fn f(){let d=wrap(&make());}fn main(){}
