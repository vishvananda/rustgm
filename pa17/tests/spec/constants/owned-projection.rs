struct S{n:usize}impl Drop for S{fn drop(&mut self){}}const C:usize=S{n:3}.n;fn main(){let _=C;}
