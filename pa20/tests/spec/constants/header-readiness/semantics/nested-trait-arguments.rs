fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}

trait A<const N:usize>{const C:usize=N;}
trait C{const D:usize=3;}
trait B{fn value()->usize;}
struct X<const N:usize>;
impl B for X<{<u8 as A<{<u16 as C>::D}>>::C}>{fn value()->usize{3}}impl<const N:usize>A<N>for u8{}
impl C for u16{}
fn main(){check(<X<3>as B>::value()==3);}