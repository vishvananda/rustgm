fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct X;impl PartialEq for X{fn eq(&self,_:&Self)->bool{true}}fn need<T:Eq+?Sized>(_:&T){}fn main(){let a:[X;0]=[];need(&a);}
