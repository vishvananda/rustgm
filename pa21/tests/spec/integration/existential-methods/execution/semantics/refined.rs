fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait Make{fn make()->impl Copy;}impl Make for u8{fn make()->u16{37}}fn main(){let value:u16=<u8 as Make>::make();check(value==37);}
