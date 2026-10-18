fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}

trait A{const C:usize=3;}
trait B{fn value()->usize;}
struct X<const N:usize>;
impl A for u8{}
impl B for X<{<u8 as A>::C}>{fn value()->usize{3}}
impl B for X<3>{fn value()->usize{4}}
fn main(){}