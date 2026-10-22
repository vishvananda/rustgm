fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct Z;impl PartialEq for Z{fn eq(&self,_:&Self)->bool{true}}fn main(){let a=[Z,Z,Z];let b=[Z,Z,Z];check(a==b);let c:[Z;0]=[];let d:[Z;0]=[];check(c==d);}
