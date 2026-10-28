#[derive(Clone)]struct Wrap<T>(T);struct Leaf;fn main(){let _=Wrap(Leaf).clone();}
