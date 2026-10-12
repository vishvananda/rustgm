fn f()->u8{let mut f:fn()->u8=a; f=loop{if true{break a;}break b;};f()} fn a()->u8{1} fn b()->u8{2}
