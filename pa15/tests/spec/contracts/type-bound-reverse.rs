fn f<'a,'b>(x:&'a u8)->&'b u8 where &'b u8:'a{x}
