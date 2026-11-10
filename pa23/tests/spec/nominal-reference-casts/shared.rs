fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct Tail<T:?Sized>{tag:u8,tail:T}trait Read{fn get(&self)->u8;}impl Read for u8{fn get(&self)->u8{*self}}
fn main(){let x=Tail{tag:7,tail:9u8};let s=&x as &Tail<dyn Read>;check(s.tag==7);check(s.tail.get()==9);}
