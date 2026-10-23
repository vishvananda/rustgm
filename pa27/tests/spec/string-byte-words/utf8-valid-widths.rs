const fn view(v:&[u8])->&str{unsafe{&*(v as *const [u8] as *const str)}}
static B:[u8;11]=[65,0,0xc2,0xa2,0xe2,0x82,0xac,0xf0,0x90,0x8d,0x88];const S:&str=view(&B);const N:u16=unsafe{let p=S as *const str as *const u8;*p as u16+*p.add(3) as u16+*p.add(6) as u16+*p.add(10) as u16};fn main(){if N!=535{let _=fail(0);}}
fn fail(x:u8)->u8{1/x}
