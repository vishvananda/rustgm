const fn view(v:&[u8])->&str{unsafe{&*(v as *const [u8] as *const str)}}
const S:&str=view(&[0xc3u8]);fn main(){let _=S;}
