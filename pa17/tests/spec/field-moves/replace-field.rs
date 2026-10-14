struct Leaf;impl Drop for Leaf{fn drop(&mut self){}}struct Outer(Leaf,u8);impl Drop for Outer{fn drop(&mut self){}}fn f(mut x:Outer){x.0=Leaf;}fn main(){}
