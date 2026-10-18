fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}

trait A{const C:usize=<u16 as B<3>>::C;}
trait B<const N:usize>{const C:usize;}
impl B<{<u8 as A>::C}>for u16{const C:usize=7;}
impl A for u8{const C:usize=3;}
fn main(){check(<u8 as A>::C==3&&<u16 as B<3>>::C==7);}