fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn main(){let b=1.25f32;check(<f32 as core::ops::Add<&f32>>::add(3.5,&b)==4.75);check(<f32 as core::ops::Sub<&f32>>::sub(3.5,&b)==2.25);}
