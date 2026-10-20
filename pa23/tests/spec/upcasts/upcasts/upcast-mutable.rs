fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait A{fn a(&mut self);}trait B:A{}impl A for u8{fn a(&mut self){*self+=2;}}impl B for u8{}fn base(x:&mut dyn B)->&mut dyn A{x}fn main(){let mut v=7u8;let p:&mut dyn B=&mut v;base(p).a();check(v==9);}
