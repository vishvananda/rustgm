trait Apply:for<'a>Fn(&'a u8)->&'a u8{}impl<F:for<'a>Fn(&'a u8)->&'a u8>Apply for F{}fn use_it<'a>(x:&'a u8){let f=|_:&u8|x;let _p:&dyn Apply=&f;}fn main(){}
