fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait A{fn f(self)->u8;}impl A for f32{fn f(self)->u8{7}}fn main(){let x=7.0.f();check(x==7);}