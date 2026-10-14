fn f<'a,'b:'c,'c>(x:&'a u8)->&'c u8 where (&'a u8,):'b{x}
