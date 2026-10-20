fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait Make{fn make()->impl Copy;}impl Make for u8{fn make()->u16{79}}impl Make for u16{fn make()->bool{true}}fn main(){let x:u16=<u8 as Make>::make();let y:bool=<u16 as Make>::make();check(x==79);check(y);}
