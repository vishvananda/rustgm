fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait A{fn f(&self)->u8;}struct W(*const u8,u8);unsafe impl Send for W{}impl A for W{fn f(&self)->u8{self.1}}fn make(x:*const u8)->impl A{W(x,11)}fn main(){let n=3u8;let x=make(&n as *const u8);let p:&(dyn A+Send)=&x;check(p.f()==11);}
