trait Value{fn value(&self)->u8;}impl Value for u8{fn value(&self)->u8{*self}}fn f()->impl Copy{7u8}fn main(){let _=f().value();}
