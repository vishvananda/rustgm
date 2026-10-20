fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait A{fn a(&mut self);}impl A for u64{fn a(&mut self){*self+=2;}}#[repr(C)]struct W<T:?Sized>(u8,T);fn change(x:&mut W<dyn A>){x.1.a();x.0+=3;}fn main(){let mut w=W(2,1000u64);change(&mut w);check(w.0==5&&w.1==1002);}
