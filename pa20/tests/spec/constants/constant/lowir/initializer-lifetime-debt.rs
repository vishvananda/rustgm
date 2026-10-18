struct S<'a>(&'a u8);impl<'a>S<'a>{const C:&'static &'a u8=&&7;}fn main(){}
