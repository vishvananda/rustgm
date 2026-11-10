fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct Tail<T:?Sized>{tag:u8,tail:T}trait Read{fn get(&self)->u8;fn set(&mut self,v:u8);}impl Read for u8{fn get(&self)->u8{*self}fn set(&mut self,v:u8){*self=v;}}fn main(){let mut x=Tail{tag:7,tail:9u8};let s=&mut x as &mut Tail<dyn Read>;s.tail.set(23);check(s.tail.get()==23);check(x.tail==23);}
