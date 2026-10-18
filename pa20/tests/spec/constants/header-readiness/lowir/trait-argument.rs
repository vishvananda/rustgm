fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}

trait A{const C:usize=3;}
trait B<const N:usize>{fn value()->usize;}
impl B<{<u8 as A>::C}>for u16{fn value()->usize{3}}
impl A for u8{}
fn main(){check(<u16 as B<3>>::value()==3);}