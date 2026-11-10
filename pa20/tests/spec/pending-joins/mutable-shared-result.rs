fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait Take<'a,R>{type Out;fn take(self,r:R)->Self::Out;}impl<'a> Take<'a,u32> for &'a mut u8{type Out=&'a mut u8;fn take(self,_:u32)->Self::Out{self}}fn choose<'a,T:Take<'a,u32,Out=&'a mut u8>>(x:T,y:&'a u8,k:bool)->&'a mut u8{let value=if k{x.take(1)}else{y};value}fn main(){}
