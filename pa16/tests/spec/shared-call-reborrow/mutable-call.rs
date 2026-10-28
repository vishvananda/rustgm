fn read(x:&u8)->u8{*x}
fn take(_: &mut u8){} fn f(r:&mut u8){take(r);take(r);}
