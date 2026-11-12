fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait It{type Item;}
impl It for u8{type Item=u16;}
fn by<T,F:FnMut(&T)->bool>(x:T,mut f:F)->bool{f(&x)}
fn always<S>(_:&S)->bool{true}
fn pick<I:It>(x:I::Item)->bool{by(x,always)}
fn main(){check(pick::<u8>(7));}
