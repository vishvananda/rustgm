fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait Read{type Output;fn read(self)->Self::Output;}struct Token(u8);impl<'a> Read for &'a Token{type Output=&'a Token;fn read(self)->Self::Output{self}}fn run<I:Read,F:FnMut(I::Output)->bool>(i:I,mut f:F)->bool{f(i.read())}fn consume(_token:Token){}fn main(){let x=Token(7);let _=run(&x,|&value|{consume(value);true});}
