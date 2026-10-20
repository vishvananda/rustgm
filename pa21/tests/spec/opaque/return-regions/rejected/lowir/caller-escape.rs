trait Value{fn value(&self)->u8;}impl<'a>Value for &'a u8{fn value(&self)->u8{**self}}fn f<'a>(x:&'a u8)->impl Value{x}fn main(){let value;{let local=7;value=f(&local);}let _=value.value();}
