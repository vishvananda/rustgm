fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait A{fn a(&self)->u16;}trait B:A{}impl A for u64{fn a(&self)->u16{*self as u16}}impl B for u64{}#[repr(C)]struct W<T:?Sized>(u8,T);fn base(x:&W<dyn B>)->&W<dyn A>{x}fn main(){let w=W(2,1000u64);let p=base(&w);check(p.1.a()==1000&&p.0==2);}
