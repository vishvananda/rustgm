fn read(x:&u8)->u8{*x}
fn borrow<T>(_:&T){}
fn consume<T>(_:T){}
fn f(r:&u8){let s:&u8=r;read(s);read(r);}
