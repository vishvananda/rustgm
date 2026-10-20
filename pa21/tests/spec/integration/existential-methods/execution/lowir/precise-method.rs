trait Make{fn make<'a>(x:&'a u8)->impl Copy+use<Self>;}impl Make for u8{fn make<'a>(x:&'a u8)->impl Copy+use<>{7u8}}fn main(){let value;{let x=1;value=<u8 as Make>::make(&x);}let _=(value,value);}
