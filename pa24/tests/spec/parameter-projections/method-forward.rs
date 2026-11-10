fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait Read{type Output;fn read(self)->Self::Output;}impl<'a> Read for &'a u8{type Output=&'a u8;fn read(self)->&'a u8{self}}trait Sequence:Read{fn run<P:FnMut(Self::Output)->bool>(self,mut f:P)->bool where Self:Sized{f(self.read())}}impl<'a> Sequence for &'a u8{}fn main(){let x=7u8;check((&x).run(|&c|c!=9u8));}
