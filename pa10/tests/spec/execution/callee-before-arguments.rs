fn a(x:u64)->u64{x+1} fn b(x:u64)->u64{x+20} fn entry()->u64{let mut f:fn(u64)->u64=a;f({f=b;1})}
