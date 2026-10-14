struct S;impl Drop for S{fn drop(&mut self){}}const C:[S;0]=[S;0];fn main(){let _=C;}
