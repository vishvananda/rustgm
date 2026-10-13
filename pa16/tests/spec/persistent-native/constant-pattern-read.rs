fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
const N:u8=7;fn main(){match 7u8{N=>{let _=N;},_=>()}}
