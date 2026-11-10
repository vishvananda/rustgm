struct Tail<T:?Sized>{tag:u8,tail:T}trait Read{fn get(&self)->u8;}impl Read for u8{fn get(&self)->u8{*self}}
impl<'a>Read for &'a u8{fn get(&self)->u8{**self}}fn cast<'a>(x:&'a Tail<&'a u8>)->&'a Tail<dyn Read+'static>{x as &'a Tail<dyn Read+'static>}fn main(){}
