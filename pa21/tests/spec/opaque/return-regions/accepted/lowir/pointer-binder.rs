trait Value{fn value(&self)->u8;}impl<'a>Value for &'a u8{fn value(&self)->u8{**self}}fn sink(_: &u8){}fn f()->impl Copy+use<>{sink as fn(&u8)}fn main(){let value=f();let _=(value,value);}
