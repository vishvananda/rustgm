fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait Collect<A>{fn collect_from(a:A)->Self;}
struct V<T>(T);
impl<T> Collect<T> for V<T>{fn collect_from(a:T)->Self{V(a)}}
fn gather<C:Collect<A>,A>(a:A)->C{C::collect_from(a)}
struct B{instance:u32}
impl B{fn get(&self)->u32{self.instance}}
fn make<T>(a:T)->V<T>{V(a)}
trait Other{}impl<T> Collect<T> for Option<T>{fn collect_from(a:T)->Self{Some(a)}}
fn main(){let b=B{instance:3};let v=gather(&b);let _=v;}
