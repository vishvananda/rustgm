fn f()->u8{take(loop{if true{break a;}break b;})} fn take(f:fn()->u8)->u8{f()} fn a()->u8{1} fn b()->u8{2}
