fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait Pair<'a,'b>{fn read(&self,x:&'a u8,y:&'b u8)->u8;}impl<'a,'b>Pair<'a,'b> for u8{fn read(&self,x:&'a u8,y:&'b u8)->u8{*self+*x+*y}}fn f()->impl for<'a,'b>Pair<'a,'b>{7u8}fn main(){let a=2;let b=3;check(f().read(&a,&b)==12);}
