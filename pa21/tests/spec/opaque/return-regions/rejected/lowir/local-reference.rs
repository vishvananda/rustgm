trait Value{fn value(&self)->u8;}impl<'a>Value for &'a u8{fn value(&self)->u8{**self}}fn f()->impl Value{let x=7;&x}fn main(){}
