fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait Ord{fn min(self,o:Self)->Self;}struct K(u8);impl Ord for K{fn min(self,o:Self)->Self{o}}fn main(){check(K(1).min(K(2)).0==2)}