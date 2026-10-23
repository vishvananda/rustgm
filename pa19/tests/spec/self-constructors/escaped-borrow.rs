struct S<'a>(&'a u8);impl<'a>S<'a>{fn bad()->Self{let n=7;Self(&n)}}fn main(){}
