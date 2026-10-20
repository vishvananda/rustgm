trait Value{fn value(&self)->u8;}impl Value for u8{fn value(&self)->u8{*self}}trait A{fn f()->impl Copy;}impl A for u8{fn f()->impl Copy+Value{7u8}}fn bad<T:A>(){let _=T::f().value();}fn main(){}
