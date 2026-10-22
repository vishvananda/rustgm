static S:u32=37;#[repr(transparent)]#[derive(Copy,Clone)]struct R(&'static u32);union U{o:Option<R>,p:&'static u32}const N:Option<R>=unsafe{(U{p:&S}).o};fn main(){check(match N{Some(R(v))=>*v==37,None=>false})}
fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
