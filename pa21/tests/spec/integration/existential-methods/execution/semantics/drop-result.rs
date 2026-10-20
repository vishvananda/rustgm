fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait Value{}struct S;impl Value for S{}impl Drop for S{fn drop(&mut self){check(false);}}trait Make{fn make()->impl Value;}impl Make for u8{fn make()->impl Value{S}}fn main(){let _value=<u8 as Make>::make();}
