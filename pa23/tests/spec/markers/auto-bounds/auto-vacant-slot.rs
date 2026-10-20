fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait A{fn f(&self)->u8 where Self:Send{7}fn g(&self)->u8{9}}struct W(*const u8);impl A for W{}fn main(){let w=W(0usize as *const u8);let p:&dyn A=&w;check(p.g()==9);}
