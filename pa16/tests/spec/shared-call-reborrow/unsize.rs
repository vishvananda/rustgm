fn read(x:&u8)->u8{*x}
fn borrow<T>(_:&T){}
fn consume<T>(_:T){}
fn take(_: &[u8]){} fn f(r:&[u8;2]){take(r);take(r);}
