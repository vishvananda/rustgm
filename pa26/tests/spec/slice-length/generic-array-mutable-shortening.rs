fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn view<'a:'b,'b,'v,const N:usize>(x:&'a mut[&'v u8;N])->&'b mut[&'v u8]{let _=x.len();x}fn main(){let v=7;let mut a=[&v;2];let s=view(&mut a);check(s.len()==2);check(*s[0]==7);}
