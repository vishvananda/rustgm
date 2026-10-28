fn read(x:&u8)->u8{*x}
fn borrow<T>(_:&T){}
fn consume<T>(_:T){}
fn f<'a>()->&'a u8{let x=1;let r=&x;r}
