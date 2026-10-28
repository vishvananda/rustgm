fn read(x:&u8)->u8{*x}
fn borrow<T>(_:&T){}
fn consume<T>(_:T){}
fn f(r:&u8,b:bool){read(match b{true=>r,false=>r});read(r);}
