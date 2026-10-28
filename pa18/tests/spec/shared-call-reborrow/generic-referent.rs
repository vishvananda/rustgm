fn read(x:&u8)->u8{*x}
fn borrow<T>(_:&T){}
fn consume<T>(_:T){}
fn f<T>(r:&T){borrow(r);borrow(r);}
