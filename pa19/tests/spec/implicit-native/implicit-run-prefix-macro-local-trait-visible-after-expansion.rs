fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
macro_rules! enable{()=>{trait A{fn f(self)->u8;}impl A for u8{fn f(self)->u8{7}}}}fn main(){enable!();let x=u8::f(3);check(x==7);}