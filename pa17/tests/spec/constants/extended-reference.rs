fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}struct S{n:usize}impl Drop for S{fn drop(&mut self){check(false);}}const R:&S=&S{n:5};fn main(){check(R.n==5);}
