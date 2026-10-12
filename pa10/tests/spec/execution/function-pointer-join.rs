fn a()->u64{7} fn b()->u64{8} fn entry()->u64{let f:fn()->u64=loop{if true{break a;}break b;};f()}
