fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}

trait A{const C:usize;}
struct X<const N:usize>;
impl A for X<{<u8 as A>::C}>{const C:usize=4;}
impl A for u8{const C:usize=3;}
fn main(){check(<X<3>as A>::C==4);}