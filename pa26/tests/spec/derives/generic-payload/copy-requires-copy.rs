#[derive(Clone)]struct Leaf;#[derive(Clone,Copy)]struct Wrap<T>(T);fn copy<T:Copy>(_:T){}fn main(){copy(Wrap(Leaf));}
