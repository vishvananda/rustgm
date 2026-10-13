fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
const S:&str="héllo";fn id<'a>(s:&'a str)->&'a str{s}fn main(){let a=id(S);let b=id("🦀");let _=(a,b);check(true);}
