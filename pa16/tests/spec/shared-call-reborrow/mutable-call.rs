fn read(x:&u8)->u8{*x}
fn borrow<T>(_:&T){}
fn consume<T>(_:T){}
fn take(_: &mut u8){} fn f(r:&mut u8){take(r);take(r);}
