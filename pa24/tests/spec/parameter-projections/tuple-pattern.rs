fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait Read{type Output;fn read(self)->Self::Output;}impl<'a> Read for &'a (u8,u16){type Output=&'a(u8,u16);fn read(self)->Self::Output{self}}fn run<I:Read,F:FnMut(I::Output)->bool>(i:I,mut f:F)->bool{f(i.read())}fn main(){let x=(7u8,9u16);check(run(&x,|&(a,b)|a==7&&b==9));}
