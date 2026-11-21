fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait Collect<A>{fn collect_from(a:A)->Self;}
struct V<T>(T);
impl<T> Collect<T> for V<T>{fn collect_from(a:T)->Self{V(a)}}
fn gather<C:Collect<A>,A>(a:A)->C{C::collect_from(a)}
fn by_key<T,K,F:FnMut(&T)->K>(v:&V<T>,mut f:F)->K{f(&v.0)}
struct B{instance:u32}
fn main(){let b=B{instance:3};let v:V<_>=gather(&b);check(v.0.instance==3);}
