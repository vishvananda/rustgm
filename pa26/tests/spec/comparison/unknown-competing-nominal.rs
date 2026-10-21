fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct S;impl PartialEq<S> for u8{fn eq(&self,_:&S)->bool{true}}fn make<T>()->T{loop{}}fn main(){let _=7u8.eq(&make());}
