fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn id(c:char)->char{c}fn main(){let a=[char::MIN,'x',char::MAX];let b=a;check(b[0] as u32==0);check(b[1] as u32==120);check(b[2] as u32==1114111);check(id(a[2])=='\u{10ffff}');let p=&a[2];check(*p=='\u{10ffff}');check(char::MAX as u8==255);check(char::MAX as u64==1114111);}
