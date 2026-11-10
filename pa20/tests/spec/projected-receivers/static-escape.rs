fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait Read<'a>{fn read(&self)->&'a u8;}impl<'a> Read<'a> for u8{fn read(&self)->&'a u8{&7}}trait Float<'a>{type Int:Read<'a>;}struct F;impl<'a> Float<'a> for F{type Int=u8;}fn conv<'a,T:Float<'a>>(x:&T::Int)->&'static u8{x.read()}fn main(){}
