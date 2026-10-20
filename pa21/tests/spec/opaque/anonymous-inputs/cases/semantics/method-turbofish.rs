trait Value{fn value(&self)->u8;}impl Value for u8{fn value(&self)->u8{*self}}struct S;impl S{fn f(&self,x:impl Value)->u8{x.value()}}fn main(){let _=S.f::<u8>(7u8);}
