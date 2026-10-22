fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn eq<T:Eq>(_:T){}fn main(){eq([1u8]);check([1u8]==[1u8]);}
