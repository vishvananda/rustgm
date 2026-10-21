trait Apply:for<'a>Fn(&'a u8)->u8{}impl<F:for<'a>Fn(&'a u8)->u8>Apply for F{}fn run(f:&dyn Apply)->u8{f(&0)}fn use_it<'a>(r:&'a u8){let f=|_:&'a u8|7u8;let _p:&dyn Apply=&f;let _=r;}fn main(){}
