fn read(x:&u8)->u8{*x}
fn borrow<T>(_:&T){}
fn consume<T>(_:T){}
fn copied<T:Copy>(_:T){} fn f(r:&u8){copied(r);}
