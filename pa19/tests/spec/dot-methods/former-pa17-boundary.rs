fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct D;impl Drop for D{fn drop(&mut self){}}impl D{fn touch(&self){}}fn main(){let d=D;d.touch();}
