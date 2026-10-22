fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn view<'a:'b,'b:'c,'c,const N:usize>(x:&'c[&'a u8;N])->&'c[&'b u8]{let _=x.len();x}fn main(){let v=7;let a=[&v;2];let s=view(&a);check(s.len()==2);check(*s[0]==7);}
