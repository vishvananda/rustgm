fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
const R:&&str=&&"🦀";fn id<'a>(s:&'a str)->&'a str{s}fn main(){let _=id(*R);}
