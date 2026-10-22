fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn pick<'s,'t>()->for<'a,'b>fn(&'a &'s u8,&'b &'t mut u8)->bool{<&'s u8 as PartialEq<&'t mut u8>>::eq}fn main(){let a=3u8;let mut b=3u8;check(pick()(&&a,&&mut b));}
