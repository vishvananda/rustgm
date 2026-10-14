struct Leaf;impl Drop for Leaf{fn drop(&mut self){}}struct Outer(Leaf,u8);impl Drop for Outer{fn drop(&mut self){}}fn f(x:[Outer;2]){let [Outer(y,_),..]=x;}fn main(){}
