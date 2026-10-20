trait Value{fn value(&self)->u8;}impl<'a>Value for &'a u8{fn value(&self)->u8{**self}}fn f<'a:'b,'b>(x:&'a mut &'a u8,y:&'b mut &'b u8,c:bool)->impl Sized{if c{x}else{y}}fn main(){}
