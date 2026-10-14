type A<'a,'b> where &'a u8:'b=&'a u8;fn f<'x>(x:&'x u8)->A<'x,'static>{x}
