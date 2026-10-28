fn read(x:&u8)->u8{*x}
fn borrow<T>(_:&T){}
fn consume<T>(_:T){}
struct N; fn take(_:N){} fn f(n:N){take(n);take(n);}
