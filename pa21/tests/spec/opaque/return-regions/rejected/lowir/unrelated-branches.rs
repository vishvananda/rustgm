trait Value{fn value(&self)->u8;}impl<'a>Value for &'a u8{fn value(&self)->u8{**self}}fn f<'a,'b>(x:&'a u8,y:&'b u8,c:bool)->impl Value{if c{x}else{y}}fn main(){}
