fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn f<T:Ord>(_:T){}fn main(){f(core::sync::atomic::Ordering::Relaxed)}