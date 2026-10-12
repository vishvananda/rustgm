fn f()->u8{let f:fn()->u8=loop{if true{break a;}break b;};f()} fn a()->u8{1} fn b()->u8{2}
