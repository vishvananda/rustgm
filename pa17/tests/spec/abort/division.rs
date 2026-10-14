fn fail(x:u8)->u8{1/x}struct D;impl Drop for D{fn drop(&mut self){}}fn main(){let _d=D;let _=fail(0);}
