fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
const N:u8=7;struct S;impl S{const P:u8=match 7{N=>1,_=>2};const C:u8=if true{1}else{Self::P};}enum E{A=match 1{S::C=>3,_=>4}}fn main(){check(E::A as u8==3);}
