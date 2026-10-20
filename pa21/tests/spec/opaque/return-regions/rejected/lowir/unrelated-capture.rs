trait Value{fn value(&self)->u8;}impl<'a>Value for &'a u8{fn value(&self)->u8{**self}}fn f<'a,'b>(_:&'a u8,x:&'b u8)->impl Value+use<'a>{x}fn main(){}
