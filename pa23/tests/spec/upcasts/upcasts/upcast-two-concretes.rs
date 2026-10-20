fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait A{fn a(&self)->u8;}trait B:A{}impl A for u8{fn a(&self)->u8{*self}}impl A for u16{fn a(&self)->u8{(*self+2)as u8}}impl B for u8{}impl B for u16{}fn base(x:&dyn B)->&dyn A{x}fn main(){let a=7u8;let b=9u16;check(base(&a).a()==7&&base(&b).a()==11);}
