const fn view(v:&[u8])->&str{unsafe{&*(v as *const [u8] as *const str)}}
const S:&str={let b=[65u8];unsafe{&*((&b as &[u8]) as *const [u8] as *const str)}};fn main(){let _=S;}
