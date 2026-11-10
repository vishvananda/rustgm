fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait Sequence{type Item;fn item(&self)->Self::Item;fn run<P:FnMut(Self::Item)->bool>(&self,mut f:P)->bool{f(self.item())}}struct S<'a>(&'a u8);impl<'a> Sequence for S<'a>{type Item=&'a u8;fn item(&self)->&'a u8{self.0}}fn main(){let x=7u8;check(S(&x).run(|&c:&u8|c!=9u8));}
