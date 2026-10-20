trait Value{fn value(&self)->u8;}impl Value for u8{fn value(&self)->u8{*self}}fn f(x:impl Value)->u8{x.value()}fn main(){let _=f(true);}
