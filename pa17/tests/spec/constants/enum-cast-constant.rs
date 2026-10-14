enum E{A=3}impl Drop for E{fn drop(&mut self){}}const C:usize=E::A as usize;fn main(){let _=C;}
