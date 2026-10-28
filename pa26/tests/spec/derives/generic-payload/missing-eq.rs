struct Leaf;impl PartialEq for Leaf{fn eq(&self,_:&Self)->bool{true}}#[derive(PartialEq,Eq)]struct Wrap<T>(T);fn equal<T:Eq>(_:T){}fn main(){equal(Wrap(Leaf));}
