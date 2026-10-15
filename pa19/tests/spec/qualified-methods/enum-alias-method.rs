fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
enum E<T>{V(T)}type A<T>=E<T>;impl<T>E<T>{fn get(self)->T{let E::V(x)=self;x}}fn main(){check(A::get(A::V(517u16))==517);}