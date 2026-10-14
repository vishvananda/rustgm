struct Leaf;impl Drop for Leaf{fn drop(&mut self){}}struct Outer(Leaf,u8);impl Drop for Outer{fn drop(&mut self){}}fn f(x:(Outer,u8)){let y=x.0;}fn main(){}
