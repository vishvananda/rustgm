struct Tail<T:?Sized>{tag:u8,tail:T}trait Read{fn get(&self)->u8;}impl Read for u8{fn get(&self)->u8{*self}}
fn cast<T>(x:&Tail<T>)->&Tail<dyn Read>{x as &Tail<dyn Read>}fn main(){}
