fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn main(){let c=core::cmp::Ordering::Less;let a=core::sync::atomic::Ordering::Relaxed;check(c as i8 == -1);check(core::cmp::Ordering::Equal as i8==0);check(core::cmp::Ordering::Greater as i8==1);check(match a{core::sync::atomic::Ordering::Relaxed=>true,_=>false});}
