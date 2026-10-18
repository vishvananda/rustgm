fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}

trait B{fn value()->usize;}
struct X<const N:usize>;
trait P{}
struct S<T>(T);
trait A{const C:usize;}
impl B for X<{<S<u8>as A>::C}>{fn value()->usize{3}}impl<T:P>A for S<T>{const C:usize=3;}
impl P for u8{}
fn main(){check(<X<3>as B>::value()==3);}