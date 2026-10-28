#[derive(PartialEq)]struct Leaf;#[derive(PartialEq,PartialOrd)]struct Wrap<T>(T);fn main(){let _=Wrap(Leaf).partial_cmp(&Wrap(Leaf));}
