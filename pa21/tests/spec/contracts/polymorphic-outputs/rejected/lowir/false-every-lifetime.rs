trait All<'a>:'a{}impl<'a,'b:'a>All<'a> for &'b u8{}fn f<'b>(x:&'b u8)->impl for<'a>All<'a>{x}fn main(){}
