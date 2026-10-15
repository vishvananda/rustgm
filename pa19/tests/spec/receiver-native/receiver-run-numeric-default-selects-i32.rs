fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait A{fn f(self)->u8;}impl A for u8{fn f(self)->u8{7}}impl A for i32{fn f(self)->u8{9}}fn main(){let x=7.f();check(x==9);}