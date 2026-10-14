fn f<'a,'b>(x:&'a u8)->&'b u8 where *const &'a u8:'b{x}
