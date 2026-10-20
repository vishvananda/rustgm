fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait Make{fn make()->(impl Copy,impl Copy);}impl Make for u8{fn make()->(u8,bool){(59,true)}}fn main(){let pair=<u8 as Make>::make();check(pair.0==59);check(pair.1);}
