fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
const S:&str=&*"héllo";fn id(s:&str)->&str{s}fn main(){let _=id(S);let _=id(&*"🦀");}
