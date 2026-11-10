fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct Tail<T:?Sized>{tag:u8,tail:T}trait Read{fn get(&self)->u8;}impl Read for u8{fn get(&self)->u8{*self}}
impl<'a>Read for &'a u8{fn get(&self)->u8{**self}}fn cast<'a,'b>(x:&'b Tail<&'a u8>)->&'b Tail<dyn Read+'a>{x as &'b Tail<dyn Read+'a>}fn main(){let v=9u8;let x=Tail{tag:7,tail:&v};let s=cast(&x);check(s.tail.get()==9);}
