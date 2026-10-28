fn read(x:&u8)->u8{*x}
fn take(_: &[u8]){} fn f(r:&[u8;2]){take(r);take(r);}
