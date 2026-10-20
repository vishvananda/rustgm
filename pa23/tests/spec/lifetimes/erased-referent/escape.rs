trait T{}impl<'b>T for &'b u8{}fn f<'a,'b>(x:&'a &'b u8)->&'a (dyn T+'static){x}fn main(){}
