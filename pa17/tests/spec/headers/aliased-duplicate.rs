struct S;type A=S;impl Drop for S{fn drop(&mut self){}}impl Drop for A{fn drop(&mut self){}}fn main(){}
