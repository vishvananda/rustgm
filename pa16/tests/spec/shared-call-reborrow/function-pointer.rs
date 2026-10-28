fn read(x:&u8)->u8{*x}
fn borrow<T>(_:&T){}
fn consume<T>(_:T){}
fn f(r:&u8){let g:fn(&u8)->u8=read;g(r);g(r);}
